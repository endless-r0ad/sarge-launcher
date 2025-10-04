<script setup lang="ts">

  import ServerRow from '@/components/ServerRow.vue'
  import Modal from '@/components/Modal.vue'
  import Loading from '@/components/Loading.vue'
  import { invoke } from '@tauri-apps/api/core'
  import { info } from '@tauri-apps/plugin-log'
  import { ensureError, newCustomServer, validServerAddress, validIp } from '@/utils/util'
  import { type Quake3Server } from '@/models/server'
  import { type MasterServer } from '@/models/master'
  import { useVirtualScroll } from '@/composables/virtualscroll'
  import { useClickRow } from '@/composables/clickrow'
  import { useLevelshot } from '@/composables/levelshot'
  import { useConfig } from '@/composables/config'
  import { useAppData } from '@/composables/appdata'
  import { useClient } from '@/composables/client'
  import { watch, nextTick, defineEmits, ref, computed, onMounted, onActivated, onDeactivated } from 'vue'
  
  const emit = defineEmits<{spawnQuake: [string[]], emitComponentName: [string], errorAlert: [string], infoAlert: [string]}>()

  const componentName = ref('Server Browser')

  const handleKeyUp = (event: KeyboardEvent) => {
    if (event.key == 'Alt') { altKeyHeld.value = false } 
  }
  const handleKeyDown = (event: KeyboardEvent) => {
    if (event.key == 'Alt') { altKeyHeld.value = true } 
    if (event.key == 'Escape') { escapeButton() }
  }

  onActivated(() => {
    document.addEventListener('keydown', handleKeyDown)
    document.addEventListener('keyup', handleKeyUp)
    emit('emitComponentName', componentName.value)
  }) 

  onDeactivated(() => {
    document.removeEventListener('keydown', handleKeyDown)
    document.removeEventListener('keyup', handleKeyUp)
  })

  onMounted(async () => {
    emit('emitComponentName', componentName.value) 
    await refreshServers(true)
  })

  const { config } = useConfig()
  const { activeClient, clientGame } = useClient()

  watch(activeClient, async(newVal, oldVal) => {
    if (config.value.refresh_by_mod && newVal?.gamename != oldVal?.gamename) {
      serverDetailsLastRefresh.value = serverIPs.value.filter((x) => x.game.includes(clientGame.value!) || x.list == 'pinned' || x.list == 'trash')
      toggleShowUnreachableServers()
    }
  })

  const { appdata, addAppData, removeAppData, writeAppData } = useAppData();

  const loading = ref(false)
  const loadingEvent = ref('')
  const q3MasterProtocol= ref(68)

  const serverIPs = ref<Quake3Server[]>([])
  const serverDetails = ref<Quake3Server[]>([])
  const serverDetailsLastRefresh = ref<Quake3Server[]>([])

  function getServersByList(list: string) { return serverDetails.value.filter((s) => s.list == list) }

  function numServersByMaster(master: MasterServer) {
    return serverDetails.value.filter((s) => s.master?.address == master.address && s.master?.game == master.game).length
  }

  const { levelshots } = useLevelshot()

  const refreshingSingle = ref(false)

  async function queryMasterServers() {
    loadingEvent.value = 'querying master servers...'
    serverIPs.value = []

    try {
      serverIPs.value = await invoke('get_q3_server_ips', { q3Protocol: q3MasterProtocol.value})
      info(`${serverIPs.value.length} servers pulled from ${activeMasterServers.value.length} active master servers`)
    }
    catch(err) {
      emit('errorAlert', ensureError(err).message)
    }
  }

  async function refreshServers(fullRefresh: boolean){

    if (loading.value) { return }

    const startTime = performance.now();
    loading.value = true

    currentSort.value = ''
    sortDesc.value = false
    selectedServer.value = null      
    serverDetails.value = []
    serverDetailsLastRefresh.value = []
    searchQuery.value = ''
      
    let refreshByMod = clientGame.value && config.value.refresh_by_mod && !fullRefresh

    if (fullRefresh) {
      await queryMasterServers()
    }
      
    loadingEvent.value = `querying ${refreshByMod ? clientGame.value : 'all'} servers...`

    try {

      serverDetailsLastRefresh.value = await invoke('refresh_all_servers', 
                { 
                  allServers: serverIPs.value.filter((x) => refreshByMod ? x.game.includes(clientGame.value!) || x.list == 'trash' : true), 
                  numThreads: (config.value.server_browser_threads == 0 ? 1 : config.value.server_browser_threads),
                  timeout: config.value.server_timeout
                })
      console.log('servers refreshed - ', serverDetailsLastRefresh.value)
    }
    catch(err) {
      emit('errorAlert', ensureError(err).message)
    }

    if (fullRefresh) {
      serverIPs.value = serverDetailsLastRefresh.value
      if (activeClient.value && config.value.refresh_by_mod) {
        serverDetailsLastRefresh.value = serverDetailsLastRefresh.value.filter((x) => x.game.includes(clientGame.value!) || x.list == 'pinned' || x.list == 'trash')
      }
    }
    
    toggleShowUnreachableServers()

    loadingEvent.value = ''
    loading.value = false
    handleScroll()

    const executionTime = performance.now() - startTime;

    let logMsg = `${serverDetailsLastRefresh.value.length - trashLength.value} servers refreshed in ${parseFloat((executionTime/1000).toFixed(2))}`
    logMsg += ` seconds using ${config.value.server_browser_threads} threads and ${config.value.server_timeout}ms timeout`
    info(logMsg)
  }

  function toggleShowUnreachableServers() {
    if (config.value.show_unreachable) {
      serverDetails.value = serverDetailsLastRefresh.value
    } else {
      serverDetails.value = serverDetailsLastRefresh.value.filter((x) => x.errormessage == '' || x.list == 'pinned')
    }
  }

  async function refreshSingleServer() {
    if (selectedServer.value == null || refreshingSingle.value) { return }
    
    refreshingSingle.value = true

    try{
      let refreshed: Quake3Server = await invoke('refresh_single_server', {refreshServer: selectedServer.value, timeout: 1000})

      let splice_index = serverDetails.value.indexOf(selectedServer.value!)
      let splice_index2 = serverDetailsLastRefresh.value.indexOf(selectedServer.value!)

      serverDetails.value[splice_index] = refreshed
      serverDetailsLastRefresh.value[splice_index2] = refreshed
      selectedServer.value = refreshed

      refreshingSingle.value = false;
    }
    catch(err){
      emit('errorAlert', ensureError(err).message)
    }
  }

  const pinnedServers = computed(() => { return serverDetails.value.filter((s) => s.list == 'pinned') }) 
      
  const mainServers = computed(() => { return serverDetails.value.filter((s) => s.list == 'main') }) 

  function getMainServerIndex(s: Quake3Server) { return mainServers.value.indexOf(s) }

  const trashServers = computed(() => { return serverDetails.value.filter((s) => s.list == 'trash') }) 
  const pinnedLength = computed(() => { return pinnedServers.value.length }) 
  const mainLength = computed(() => { return mainServers.value.length }) 
  const trashLength = computed(() => { return trashServers.value.length }) 

  const { 
     translateY,
     virtualHeight,
     marginTop,
     virtualStartIndex,
     virtualEndIndex,
     handleScroll
  } = useVirtualScroll('serverTable', mainLength, pinnedLength)

  const getVirtualRows = computed(() => {
    return serverDetails.value.filter((s) => s.list == 'main').slice(virtualStartIndex.value, virtualEndIndex.value)
  })

  const addtlHeight = computed(() => {
    let extra = 0
    if (pinnedLength.value == 0) { extra += 48 }
    if (trashLength.value == 0) { extra += 48 }
    return (24 * pinnedLength.value) + extra
  })

  const showPopup = ref('')

  async function handleAddCustomPopup(closeAfterHandle: boolean) {
    if (popupInput.value == '') { return }
    if (!validServerAddress(popupInput.value)) {
      emit('errorAlert', 'not a valid IP:Port')
      popupInput.value = '', showPopup.value = '';
      return
    }
    if (appdata.value.pinned.has(popupInput.value)) {
      emit('infoAlert', 'custom server is already a pinned server')
      popupInput.value = '', showPopup.value = '';
      return
    }

    let alreadyOnMaster = serverDetailsLastRefresh.value.find((s) => s.address == popupInput.value)

    try{

      addAppData('custom', popupInput.value)
      await writeAppData()

      if (alreadyOnMaster) {
        alreadyOnMaster.list = 'pinned'
        popupInput.value = ''
        return
      } 

      let ip_and_port = popupInput.value.split(':')
      let customServer: Quake3Server = newCustomServer(ip_and_port, popupInput.value)
      
      lastSelectedServer.value = selectedServer.value
      selectedServer.value = customServer
      refreshingSingle.value = true

      serverDetails.value.push(customServer)
      serverDetailsLastRefresh.value.push(customServer) 
      
      let refreshed: Quake3Server = await invoke('refresh_single_server', {refreshServer: selectedServer.value, timeout: 1000})

      serverDetails.value[serverDetails.value.length-1] = refreshed
      serverDetailsLastRefresh.value[serverDetailsLastRefresh.value.length-1] = refreshed
      selectedServer.value = refreshed

      refreshingSingle.value = false;

      if (closeAfterHandle) {
        popupInput.value = '', showPopup.value = '';
      } else {
        popupInput.value = ''
      }

    }
    catch(err){
      emit('errorAlert', ensureError(err).message)
    }
  }

  async function handleAddTrashIpPopup(closeAfterHandle: boolean) {
    if (popupInput.value == '') { return }

    if (!validIp(popupInput.value)) {
      emit('errorAlert', 'not a valid IP')
      popupInput.value = '', showPopup.value = '';
      return
    }

    try{
      addAppData('trash_ip', popupInput.value)
      serverDetails.value.map((serv: Quake3Server) => { if (serv.ip == popupInput.value){ serv.list = 'trash' } });
      await writeAppData()
    }
    catch(err){
      emit('errorAlert', ensureError(err).message)
    }
 
    if (closeAfterHandle) {
      popupInput.value = '', showPopup.value = '';
    } else {
      popupInput.value = ''
    }
  }

  async function handleServerPasswordPopup(closeAfterHandle: boolean) {
    if (popupInput.value == '') { return }

    try {
      appdata.value.server_password = popupInput.value
      await writeAppData()
      let args = ['+set', 'fs_game', selectedServer.value!.game, '+password', popupInput.value, '+connect', selectedServer.value!.address];
      emit('spawnQuake', args)   

      if (closeAfterHandle) {
        popupInput.value = '', showPopup.value = '';
      } else {
        popupInput.value = ''
      } 
    } catch(err) {
      emit('errorAlert', ensureError(err).message)
    }
  }

  async function handleMasterServerPopup(closeAfterHandle: boolean) {
    try {
      await writeAppData()
      if (closeAfterHandle) {
        popupInput.value = '', showPopup.value = '';
      } else {
        popupInput.value = ''
      }
      await refreshServers(true)
    } catch(err) {
      emit('errorAlert', ensureError(err).message)
    }
  }

  const sortDesc = ref(false)
  const currentSort = ref('')

  function getArrowSort(column: string) {
    if (currentSort.value != column) { return 'sort-arrow-default'}
    if (sortDesc.value && currentSort.value == column) { return 'sort-arrow-desc'}
    if (!sortDesc.value && currentSort.value == column) { return 'sort-arrow-asc'}
  }

  function sortServers(column: string){
    if (currentSort.value == column || currentSort.value == ''){
      sortDesc.value = !sortDesc.value
    } else {
      sortDesc.value = true;
    }

    selectedServer.value = null
    currentSort.value = column

    if (column == 'game' || column == 'host' || column == 'map' || column == 'playersconnected' || column == 'ping' || column == 'address' ) {
      serverDetails.value.sort((a, b) => {
        if (a[column] > b[column]) {
          return ( sortDesc.value ? -1 : 1 );
        }
        if (a[column] < b[column]) {
          return ( sortDesc.value ? 1 : -1 );
        }
        return 0;
      });
    }
  }

  const selectedServer = ref<Quake3Server | null>(null)
  const lastSelectedServer = ref<Quake3Server | null>(null)

  watch(selectedServer, (_new, _old) => {
      keepSelectedDetailsOpen.value = false
    })

  const selectedServerIndex = computed(() => {
    if (selectedServer.value) {
      if (selectedServer.value.list == 'main') {
        getVirtualRows.value.indexOf(selectedServer.value)
      }     
      return getServersByList(selectedServer.value.list).indexOf(selectedServer.value)
    }
    return -1
  })

  function keySelectOutOfBound(proposedIndex: number) {
    if ((proposedIndex < 0 && selectedServer.value?.list == 'pinned') || 
        (proposedIndex > trashLength.value-1 && selectedServer.value?.list == 'trash') ||
        (proposedIndex > mainLength.value-1 && selectedServer.value?.list == 'main' && (trashLength.value == 0 || !config.value.show_trashed_servers)) ||
        (proposedIndex < 0 && selectedServer.value?.list == 'main' && pinnedLength.value == 0))
        {
        return true
        }  
    return false
  }

  function keySelect(increment: number){   
    let proposedIndex = selectedServerIndex.value + increment
      
    if (selectedServer.value == null || refreshingSingle.value || keySelectOutOfBound(proposedIndex))
    {
      return
    }

    lastSelectedServer.value = selectedServer.value

    if (proposedIndex < 0 && selectedServer.value.list == 'main'){
      selectedServer.value = pinnedServers.value[pinnedLength.value-1]!
    }
    else if (proposedIndex < 0 && selectedServer.value.list == 'trash'){
      selectedServer.value = mainServers.value[mainLength.value-1]!
    }
    else if (selectedServer.value.list == 'pinned' && proposedIndex > pinnedLength.value-1){
      selectedServer.value = mainServers.value[0]!
    }
    else if (selectedServer.value.list == 'main' && proposedIndex > mainLength.value-1){
      selectedServer.value = trashServers.value[0]!
    }
    else {
      selectedServer.value = getServersByList(selectedServer.value.list)[proposedIndex]!
    }

    if (pinnedLength.value == 0  && proposedIndex == 0 && selectedServer.value.list == 'main') {
      document.getElementById('scrollEmptyPinned')?.scrollIntoView()
      return
    }

    nextTick(() => {
      // once the real dom is updated (document.), scroll the new selected into view
      let selected = document.getElementById('selected')     
      selected?.scrollIntoView({behavior: 'instant', block: 'start', inline: 'nearest'})
      selected?.focus()
    })
  }

  function spawnQuake(){
    if (selectedServer.value == null) { return }

    try {
      let args = ['+set', 'fs_game', selectedServer.value.game, '+set', 'protocol', selectedServer.value.protocol?.toString() ?? '68', '+connect', selectedServer.value.address];
      if ('g_needpass' in selectedServer.value.othersettings && selectedServer.value.othersettings['g_needpass'] == '1') {
        showPopup.value = 'password'
        popupInput.value = appdata.value.server_password
      } else {
        emit('spawnQuake', args)
      }
    } catch(err) {
      emit('errorAlert', ensureError(err).message)
    }       
  }

  const popupInput = ref('')
  const masterServerHover = ref(false)

  const activeMasterServers = computed(() => {
    return appdata.value.masters.filter((m) => m.active);
  })

  const altKeyHeld = ref(false)
  const keepSelectedDetailsOpen = ref(false)

  watch(() => config.value.show_unreachable, (newVal, _oldVal) => {
    let search = searchQuery.value

    if (newVal) {
      serverDetails.value = serverDetailsLastRefresh.value
    } else {
      serverDetails.value = serverDetailsLastRefresh.value.filter(x => x.errormessage == '' || x.list == 'pinned')
    }

    if (search != '') {
      nextTick(() => {  // force the searchQuery watch 
        searchQuery.value = ''
        nextTick(() => {
          searchQuery.value = search
        })
      })
    }
  })

  watch(() => config.value.refresh_by_mod, (newVal, _oldVal) => {
    if (newVal && activeClient.value) {
      serverDetailsLastRefresh.value = serverIPs.value.filter((x) => x.game.includes(clientGame.value!) || x.list == 'pinned' || x.list == 'trash')
      toggleShowUnreachableServers()
    }
    if (!newVal) {
      serverDetailsLastRefresh.value = serverIPs.value
      toggleShowUnreachableServers()
    }
  })

  const searchQuery = ref('')

  watch(searchQuery, (newSearch, _old) => { 
    selectedServer.value = null
    lastSelectedServer.value = null
    sortDesc.value = false 
    currentSort.value = ''
    
    let serversFrom = serverDetailsLastRefresh.value
    let query = newSearch.toLowerCase()
    let filteredServers: Quake3Server[] = []
    
    if (!config.value.show_unreachable) {
      serversFrom = serverDetailsLastRefresh.value.filter((x) => x.errormessage == '')
    }
    
    for(let i = 0; i < serversFrom.length; i++) {
      if(serversFrom[i]!.map.toLowerCase().includes(query) ||
        serversFrom[i]!.game.toLowerCase().includes(query) ||
        serversFrom[i]!.host.toLowerCase().includes(query) ||
        serversFrom[i]!.ip.toLowerCase().includes(query)
        ) 
      {
        filteredServers.push(serversFrom[i]!);
      }
    }
    serverDetails.value = filteredServers
  })

  async function removeFromCustom(address: string) {
    let server = serverDetailsLastRefresh.value.find((s) => s.address == address)

    if (server) {
      server.list = 'main'
    }
    removeAppData('custom', address)
    writeAppData()
  }

  function escapeButton() { lastSelectedServer.value = null, selectedServer.value = null }

  function displayTrashedServers() {
    config.value.show_trashed_servers = true
  }

  const displayDetails = ref(false)

  const {
   handleClick,
   dblClickHappenedOnSameObject,
   resetDblClickTimeout,
   rightClickToSelect
  } = useClickRow(selectedServer, lastSelectedServer, displayDetails);

  async function clickServer(selectedServ: Quake3Server, _index: number, event: MouseEvent){
    let target = event.target as HTMLTextAreaElement
    
    if (target.id == 'addToListButton') { 
      let toList = ''
      if (selectedServ.list == 'main') {
        if (event.altKey) {
          toList = 'trash'
          addAppData('trash', selectedServ.address)    
        } else {
          toList = 'pinned'
          addAppData('pinned', selectedServ.address)
        }
      } 
      if (selectedServ.list == 'pinned') {
        toList = 'main'
        removeAppData('pinned', selectedServ.address)
      }
      if (selectedServ.list == 'trash') {
        toList = 'main'
        removeAppData('trash', selectedServ.address)     
      }
      
      await writeAppData()
      selectedServ.list = toList
      selectedServer.value = null

      return
    }

    handleClick(selectedServ, target)

    if (dblClickHappenedOnSameObject.value) {
      spawnQuake()
      resetDblClickTimeout()
    }
  }

</script>

<template>
    
  <div class="table-header-base no-select">
    <div class="table-header-right">
        <input class="search" type="text" placeholder="search" v-model="searchQuery"> 
        <span class="add-custom-server" :class="{'active-popup': showPopup == 'add'}" @click="showPopup='add'" />
        <span class="trash-server-button" :class="{'active-popup': showPopup == 'trash'}" @click="showPopup='trash'" />
    </div>
    <div class="table-header-left">        
      <button class="connect-button" :disabled="selectedServer == null" @click="spawnQuake();">Connect</button>            
      <button class="refresh-button" @click="refreshServers(false);">Refresh</button>
      <span class="refresh-master-button" @click="refreshServers(true);" />
    </div> 

  
    <div class="table-column-header">
      <span style="width: 3%;"></span>
      <span style="width: 11%; text-align: left;"><span class="sort-header" @click="sortServers('game');">game</span><span :class="getArrowSort('game')" @click="sortServers('game');"/></span>
      <span style="width: 3%;"></span>
      <span style="width: 36%; text-align: left;"><span class="sort-header" @click="sortServers('host');">hostname</span><span :class="getArrowSort('host')" @click="sortServers('host');" /></span>
      <span style="width: 1%;"></span>
      <span style="width: 16%; text-align: left;"><span class="sort-header" @click="sortServers('map');">map</span><span :class="getArrowSort('map')" @click="sortServers('map');"/></span>
      <span style="width: 10%; text-align: left;"><span class="sort-header" @click="sortServers('playersconnected')">players</span><span :class="getArrowSort('playersconnected')" @click="sortServers('playersconnected')"/></span>
      <span style="width: 2%;"></span>   
      <span style="width: 7%; text-align: left;"><span class="sort-header" @click="sortServers('ping');">ping</span><span :class="getArrowSort('ping')" @click="sortServers('ping');"/></span>  
      <span style="width: 16%; text-align: left;"><span class="sort-header" @click="sortServers('address');">address</span><span :class="getArrowSort('address')" @click="sortServers('address');"/></span>  
      <span style="width: 2%;"></span>     
    </div>
  </div>

  
  <div class="scrollable-container" 
      @keydown.up.prevent="keySelect(-1)" 
      @keydown.down.prevent="keySelect(1)" 
      @keydown.enter.prevent="spawnQuake()"
      @keydown.space.prevent="refreshSingleServer()"
      id="serverTable"
      ref="serverTable"
      >
    <div v-if="loading" >  
      <Loading :position="'center'" :message="loadingEvent" :size="90" />
      <div class="empty-pinned"><span><img src="../assets/icons/pin.svg" class="pin-icon"></span></div>
      <div v-for="(_, index) in 48" class="row" :style="index % 2 ? 'background-color: rgba(23, 32, 45, 0.3);' : ''" ></div>     
      <div class="empty-trash"><span>alt + <img src="../assets/icons/trash.svg" class="trash-icon"></span></div>      
    </div>
    <div v-if="!loading" :style="{ height: (virtualHeight + addtlHeight) + 'px'}">      
      <div class="main" v-bind:style="{ transform: 'translateY(' + translateY + 'px)', marginTop: marginTop + 'px' }">      
        <div v-if="pinnedLength == 0" id="scrollEmptyPinned" class="empty-pinned">
          <img src="../assets/icons/pin.svg" class="pin-icon">
        </div>
        <ServerRow v-for="(server, index) in pinnedServers" 
          class="row pinned"
          :key="server.address"      
          :id="server === selectedServer ? 'selected' : `pinned-${index}`"
          :server="server"
          :isSelected="server === selectedServer"
          :displayDetails="displayDetails"
          :refreshing="refreshingSingle"
          :altKeyHeld="altKeyHeld"
          :levelshotPath="levelshots[server.map.toLowerCase()] ?? null"
          tabindex="0"
          @click="clickServer(server, index, $event);"
          @showDetails="displayDetails = true"
          @hideDetails="displayDetails = false"
          @contextmenu.prevent="rightClickToSelect(server); refreshSingleServer();"
        />              
        <ServerRow v-for="(server, index) in getVirtualRows" 
          class="row"
          :style="getMainServerIndex(server) % 2 ? 'background-color: rgba(23, 32, 45, 0.3);' : ''"
          :key="server.address"    
          :id="server === selectedServer ? 'selected' : ''"
          :server="server"
          :isSelected="server === selectedServer"
          :displayDetails="displayDetails"
          :refreshing="refreshingSingle"
          :altKeyHeld="altKeyHeld"
          :displayDetailsOnMount="keepSelectedDetailsOpen"
          :levelshotPath="levelshots[server.map.toLowerCase()] ?? null"
          tabindex="0"
          @click="clickServer(server, index, $event);"
          @showDetails="displayDetails = true"
          @hideDetails="displayDetails = false"
          @contextmenu.prevent="rightClickToSelect(server); refreshSingleServer();"
          @detailsDisplayedOnUnmount="keepSelectedDetailsOpen = !keepSelectedDetailsOpen"
          />    
        <div v-if="config.show_trashed_servers">
          <ServerRow v-for="(server, index) in trashServers" 
          class="row trash"
          :key="server.address" 
          :id="server === selectedServer ? 'selected' : `trash-${index}`"
          :server="server"
          :isSelected="server === selectedServer"
          :displayDetails="displayDetails"
          :refreshing="refreshingSingle"
          :altKeyHeld="altKeyHeld"
          :levelshotPath="levelshots[server.map.toLowerCase()] ?? null"
          tabindex="0"
          @click="clickServer(server, index, $event);"
          @showDetails="displayDetails = true"
          @hideDetails="displayDetails = false"
          @contextmenu.prevent="rightClickToSelect(server); refreshSingleServer();"
          /> 
        </div> 
        <div v-if="!config.show_trashed_servers && trashLength > 0" class="hidden-trash" @click="displayTrashedServers()">
          ...
        </div>                
        <div v-if="trashLength == 0" class="empty-trash">
          alt + <img src="../assets/icons/trash.svg" class="trash-icon">
        </div>
      </div>    
    </div>
  </div>

  <div class="table-footer">
    <div class="table-footer-right">
      <span v-if="config.show_trashed_servers" class="footer-data-right">Servers: {{ serverDetails.length }}</span>
      <span v-else class="footer-data-right">Servers: {{ serverDetails.length - trashLength }}</span>   
    </div>
    <div class="table-footer-left">  
      <button @mouseover="masterServerHover=true"
            @mouseleave="masterServerHover=false" 
            class="refresh-button"
            @click="showPopup='masterSettings'">
        Master Servers
      </button>
      <div v-if="masterServerHover" class="footer-popup">
        <div v-for="master in appdata.masters" style="padding-right: 40px;">
          <div v-if="master.active" style="display: inline-block; width: 15%;">{{ numServersByMaster(master) }} </div>
          <div v-if="master.active" style="display: inline-block;">{{ master.game }}: {{ master.name }}</div>                 
        </div>
      </div> 
    </div>   
  </div>   


  <Teleport to="#modal">
    <Modal v-if="showPopup=='add'" :popupType="'center'" @executeModal="handleAddCustomPopup(true)" @cancelModal="popupInput = '', showPopup = ''">
      <label>Add a Custom Server</label> 
      <div>
        <input type="text" placeholder="0.0.0.0:0" v-model="popupInput" class="search" @keyup.enter="handleAddCustomPopup(false)">
        <span class="ok-button" @click="handleAddCustomPopup(true)">ok</span>
      </div>
    
      <div style="overflow: auto; max-height: 200px; margin-right: -16px;">
        <div v-for="address in appdata.custom"> 
          <button @click="removeFromCustom(address)" class="close-button"><img src="../assets/icons/x.svg" width="8px" /></button>
          <label style="font-size: 80%;">{{ address }}</label>
        </div>
      </div>
    </Modal>
          
    <Modal v-if="showPopup=='trash'" :popupType="'center'" @executeModal="handleAddTrashIpPopup(true)" @cancelModal="popupInput = '', showPopup = ''">
      <label>Trash all servers of an IP
        <div>
          <input type="text" placeholder="0.0.0.0" v-model="popupInput" class="search"  @keyup.enter="handleAddTrashIpPopup(false)">
          <span class="ok-button" @click="handleAddTrashIpPopup(true)">ok</span>
        </div>      
      </label>  
      <div style="overflow: auto; max-height: 200px; margin-right: -16px;">
        <div v-for="ip in appdata.trash_ip">
          <button @click="removeAppData('trash_ip', ip); writeAppData();" class="close-button"><img src="../assets/icons/x.svg" width="8px" /></button>
          <label style="font-size: 80%;">{{ ip }}</label>
        </div>
      </div>    
    </Modal>

    <Modal v-if="showPopup=='password'" :popupType="'center'" @executeModal="handleServerPasswordPopup(true)" @cancelModal="popupInput = '', showPopup = ''">
      <label>Enter server password
        <div>
          <input type="text" :placeholder="'password'" v-model="popupInput" class="search" @keyup.enter="handleServerPasswordPopup(true)">
          <span class="ok-button" @click="handleServerPasswordPopup(true)">ok</span>
          </div>      
      </label>  
    </Modal>
      
    <Modal v-if="showPopup=='masterSettings'" :popupType="'center'" @executeModal="handleMasterServerPopup(true)" @cancelModal="popupInput = '', showPopup = ''">   
      <h3 style="text-align: center; margin-top: -10px;">Master Servers Settings</h3>
      <div v-for="master in appdata.masters" style="height: 32px;">
        <span><input type="checkbox" v-model="master.active"></span>
        <text :style="master.unreachable ? 'color: #aaa; text-decoration: line-through;' : ''" class="ml-1">{{ master.game }}: {{ master.name }} </text>
      </div>
      <div style="height: 32px; margin: 0px 4px 0px 2px; text-align: left;">
        <text>+</text>
        <text class="ml-1">Quake 3 Master Protocol</text>
        <text class="ml-1"><input type="radio" :value="Number(68)" v-model="q3MasterProtocol">68</text>
        <text class="ml-1"><input type="radio" :value="Number(43)" v-model="q3MasterProtocol">43</text>
      </div>
      <div style="height: 32px; text-align: center;">
        <span>Refresh Master List: </span>
        <span class="refresh-master-button" @click="handleMasterServerPopup(true)"></span>
      </div>
    </Modal>
  </Teleport>

</template>
  
  
<style scoped>

  #scrollEmptyPinned {
    text-align: center; 
    color: white; 
    line-height: 48px;
    font-size: large;
    font-weight: 600;
  }

  .unfiltered {
    width: 8px;
    height: 8px;
    cursor: pointer;
    position: relative;
    margin: 3px 0 0 2px;
  }

  .pinned {
    background-color: var(--alt-bg);
  }

  .pinned:nth-of-type(even) {
    background-color: rgba(9, 61, 82, 0.5)
  }

  .pin-icon {
    height: 22px;
    margin-bottom: -4px;
  }

  .main {
    min-height: 24px;
    position: relative;
    cursor: default;
  }

  .trash {
    background-color: rgba(36, 68, 37, 0.53);
  }

  .trash:nth-of-type(even) {
    background-color: rgba(36, 68, 37, 0.25);
  }

  .trash-icon {
    height: 22px;
    margin-bottom: -2px;
  }

  .servers-tip-text {
    visibility: hidden;
    width: 120px;
    background-color: black;
    color: #fff;
    text-align: center;
    padding: 5px 0;
    border-radius: 6px;

  }

  .servers-tip:hover .servers-tip-text {
    visibility: visible;
  }

  .refresh-master-button {
    cursor: pointer;
    padding: 4px 15px;
    margin-top: 1px;
    background: url('../assets/icons/pull-arrow.svg') center center no-repeat;
  }

  .refresh-master-button:hover {    
    background-color: var(--main-bg);
    border-radius: 0.2rem;
  }

  .add-custom-server {
    padding: 4px 15px;
    margin: 0px 4px; 
    background: url('../assets/icons/plus.svg') center center no-repeat;
    background-size: 14px;
  }

  .add-custom-server:hover {    
    background-color: var(--main-bg);
    border-radius: 0.2rem;
    cursor: pointer;
  }

  .trash-server-button {
    padding: 4px 15px;  
    background: url('../assets/icons/trash.svg') center center no-repeat;
  }

  .trash-server-button:hover {    
    background-color: var(--main-bg);
    border-radius: 0.2rem;
    cursor: pointer;
  }

  .active-popup {    
    background-color: var(--main-bg);
    border-radius: 0.2rem;
    cursor: pointer;
  }

  .master-hover-right {
    float: right;
    width: 85%;
  }

  .master-hover-left {
    margin: 0 86% 0 0;
  }


  .empty-pinned {
    background-color: var(--alt-bg);
    text-align: center; 
    color: white; 
    line-height: 48px;
    font-size: large;
    font-weight: 600;
  }

  .empty-trash {
    background-color: rgba(36, 68, 37, 0.53);
    text-align: center; 
    color: white; 
    line-height: 48px;
    font-size: large;
    font-weight: 600;
  }

  .hidden-trash {
    background-color: rgba(36, 68, 37, 0.53);
    text-align: center; 
    color: white; 
    line-height: 48px;
    font-size: large;
    font-weight: 600;
    cursor: pointer;
  }

  .hidden-trash:hover {
    background-color: rgba(36, 68, 37, 0.25);
  }

  .data {
    text-align: left;
    white-space: nowrap; 
    overflow: hidden;
  }

  .close-button {
    background: rgba(0, 0, 0, 0);
    border: 1px solid var(--main-bg);
    border-radius: 0.2rem;
    margin: 0px 8px 8px 0px;
  }

  .close-button:hover {
    background-color: var(--main-bg);
    cursor: pointer;
  }

  .custom-ip {
    outline: none;
    width: 80%;
    height: 18px;
    background-color: var(--main-bg);
    color: #ccc;
    border-width: 0px;
    border-radius: 0.2rem;
    margin: 8px 0px;
    padding: 2px 0px;
    text-indent: 4px;
  }

  .custom-ip::selection {
    outline: 1px solid #333;
  }
    
</style>