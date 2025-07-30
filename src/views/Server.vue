<script setup lang="ts">

  import ServerRow from '@/components/ServerRow.vue'
  import Modal from '@/components/Modal.vue'
  import Loading from '@/components/Loading.vue'
  import { invoke } from '@tauri-apps/api/core'
  import { info } from '@tauri-apps/plugin-log'
  import { ensureError, newCustomServer, validServerAddress, validIp } from '@/utils/util';
  import { type Config, type AppData } from '@/models/config'
  import { type Q3Executable } from '@/models/client'
  import { type Quake3Server } from '@/models/server'
  import { type MasterServer } from '@/models/master'
  import { useVirtualScroll } from '@/composables/virtualscroll';
  import { useClickRow } from '@/composables/clickrow';
  import { useLevelshot } from '@/composables/levelshot'
  import { watch, nextTick, defineProps, defineEmits, ref, computed, onMounted, onActivated, onDeactivated } from 'vue'

  const props = defineProps<{ config: Config, appData: AppData, showUnreachableServers: boolean, showTrashedServers: boolean, activeClient: Q3Executable }>()
  
  const emit = defineEmits<{
    mutateConfig: [Config],
    mutateAppData: [AppData],
    spawnQuake: [string[]],
    addQ3Client: [Q3Executable],
    emitComponentName: [string],
    errorAlert: [string],
    infoAlert: [string],
  }>();

  const localAppData = ref(props.appData)

  const componentName = ref('Server Browser')

  async function emitComponentName(){
    emit('emitComponentName', componentName.value)
  }

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

  const { levelshots, syncLevelshots, levelHasLevelshot } = useLevelshot()

  const refreshingSingle = ref(false)

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
    sortDesc.value = false 
    currentSort.value = ''
      
    if (fullRefresh) {
      loadingEvent.value = 'querying master servers...'
      serverIPs.value = []
      
      try{
        serverIPs.value = await invoke('get_q3_server_ips', { q3Protocol: q3MasterProtocol.value })
        info(`${serverIPs.value.length} servers pulled from ${activeMasterServers.value.length} active master servers`)
      }
      catch(err){
        emit('errorAlert', ensureError(err).message)
      }
    }
      
    loadingEvent.value = 'querying servers...'

    try {
      await syncLevelshots(props.activeClient, false)

      serverDetailsLastRefresh.value = await invoke('refresh_all_servers', 
                { allServers: serverIPs.value, 
                  numThreads: (props.config.server_browser_threads == 0 ? 1 : props.config.server_browser_threads),
                  timeout: props.config.server_timeout
                })
    }
    catch(err) {
      emit('errorAlert', ensureError(err).message)
    }
    
    if (props.showUnreachableServers) {
      serverDetails.value = serverDetailsLastRefresh.value
    } else {
      serverDetails.value = serverDetailsLastRefresh.value.filter((x) => x.errormessage == '' || x.list == 'pinned')
    }

    loadingEvent.value = ''
    loading.value = false
    handleScroll()

    const executionTime = performance.now() - startTime;

    let logMsg = `${serverDetailsLastRefresh.value.length} servers refreshed in ${parseFloat((executionTime/1000).toFixed(2))} seconds `
    logMsg += `using ${props.config.server_browser_threads} threads and ${props.config.server_timeout}ms timeout`

    info(logMsg)
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
    if (popupInput.value != '') {
      if (validServerAddress(popupInput.value)) {
        if (!props.appData.pinned.has(popupInput.value)) {
          try{
            addCustom(popupInput.value)
  
            let ip_and_port = popupInput.value.split(':')
            let customServer: Quake3Server = newCustomServer(ip_and_port, popupInput.value)

            lastSelectedServer.value = selectedServer.value
            selectedServer.value = customServer
            refreshingSingle.value = true

            serverDetails.value.push(customServer)
            serverDetailsLastRefresh.value.push(customServer) 
            
            try{
              let refreshed: Quake3Server = await invoke('refresh_single_server', {refreshServer: selectedServer.value, timeout: 1000})

              serverDetails.value[serverDetails.value.length-1] = refreshed
              serverDetailsLastRefresh.value[serverDetailsLastRefresh.value.length-1] = refreshed
              selectedServer.value = refreshed

              refreshingSingle.value = false;
            }
            catch(err){
              emit('errorAlert', ensureError(err).message)
            }
          }
          catch(err){
            emit('errorAlert', ensureError(err).message)
          }
        } else {
          emit('infoAlert', 'custom server is already a pinned server')
        }
      } else {
        emit('errorAlert', 'not a valid IP:Port')
        popupInput.value = '', showPopup.value = '';
      }
    }
    if (closeAfterHandle) {
      popupInput.value = '', showPopup.value = '';
    } else {
      popupInput.value = ''
    }
  }

  async function handleAddTrashIpPopup(closeAfterHandle: boolean) {
    if (popupInput.value != '') {
      if (validIp(popupInput.value)) {
        try{
          addTrashIP(popupInput.value)
          serverDetails.value.map((serv: Quake3Server) => { if (serv.ip == popupInput.value){ serv.list = 'trash' } });
        }
        catch(err){
          emit('errorAlert', ensureError(err).message)
        }
      } else {
        emit('errorAlert', 'not a valid IP')
      }
    }
    if (closeAfterHandle) {
      popupInput.value = '', showPopup.value = '';
    } else {
      popupInput.value = ''
    }
  }

  async function handleServerPasswordPopup(closeAfterHandle: boolean) {
    if (popupInput.value != '') {
      localAppData.value.server_password = popupInput.value
      emit('mutateAppData', localAppData.value)
      let args = ['+set', 'fs_game', selectedServer.value!.game, '+password', popupInput.value, '+connect', selectedServer.value!.address];
      emit('spawnQuake', args)   

      if (closeAfterHandle) {
        popupInput.value = '', showPopup.value = '';
      } else {
        popupInput.value = ''
      } 
    }
  }

  function handleMasterServerPopup(closeAfterHandle: boolean) {
    emit('mutateAppData', localAppData.value)
    if (closeAfterHandle) {
      popupInput.value = '', showPopup.value = '';
    } else {
      popupInput.value = ''
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
        (proposedIndex > mainLength.value-1 && selectedServer.value?.list == 'main' && (trashLength.value == 0 || !props.showTrashedServers)) ||
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
      selectedServer.value = pinnedServers.value[pinnedLength.value-1]
    }
    else if (proposedIndex < 0 && selectedServer.value.list == 'trash'){
      selectedServer.value = mainServers.value[mainLength.value-1]
    }
    else if (selectedServer.value.list == 'pinned' && proposedIndex > pinnedLength.value-1){
      selectedServer.value = mainServers.value[0]
    }
    else if (selectedServer.value.list == 'main' && proposedIndex > mainLength.value-1){
      selectedServer.value = trashServers.value[0]
    }
    else {
      selectedServer.value = getServersByList(selectedServer.value.list)[proposedIndex]
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
    if (selectedServer.value) {
      let args = ['+set', 'fs_game', selectedServer.value.game, '+set', 'protocol', selectedServer.value.protocol?.toString() ?? '68', '+wait', '5', '+connect', selectedServer.value.address];
      if ('g_needpass' in selectedServer.value.othersettings && selectedServer.value.othersettings['g_needpass'] == '1') {
        showPopup.value = 'password'
        popupInput.value = props.appData.server_password
      } else {
        console.log('spawning with args ', args)
        emit('spawnQuake', args)
      }          
    }        
  }

  const popupInput = ref('')
  const masterServerHover = ref(false)

  const activeMasterServers = computed(() => {
    return props.appData.masters.filter((m) => m.active);
  })

  const altKeyHeld = ref(false)
  const keepSelectedDetailsOpen = ref(false)

  watch(() => props.showUnreachableServers, (newVal, _oldVal) => {
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

  const searchQuery = ref('')

  watch(searchQuery, (newSearch, _old) => { 
    selectedServer.value = null
    lastSelectedServer.value = null
    sortDesc.value = false 
    currentSort.value = ''
    
    let serversFrom = serverDetailsLastRefresh.value
    let query = newSearch.toLowerCase()
    let filteredServers = []
    
    if (!props.showUnreachableServers) {
      serversFrom = serverDetailsLastRefresh.value.filter((x) => x.errormessage == '')
    }
    
    for(let i = 0; i < serversFrom.length; i++) {
      if(serversFrom[i].map.toLowerCase().includes(query) ||
        serversFrom[i].game.toLowerCase().includes(query) ||
        serversFrom[i].host.toLowerCase().includes(query) ||
        serversFrom[i].ip.toLowerCase().includes(query)
        ) 
      {
        filteredServers.push(serversFrom[i]);
      }
    }
    serverDetails.value = filteredServers
  })

  function addCustom(address: string) { localAppData.value.custom.add(address), emit('mutateAppData', localAppData.value) }
  function removeCustom(address: string) { localAppData.value.custom.delete(address), emit('mutateAppData', localAppData.value) }

  function addTrash(address: string) { localAppData.value.trash.add(address), emit('mutateAppData', localAppData.value) }
  function removeTrash(address: string) { localAppData.value.trash.delete(address), emit('mutateAppData', localAppData.value) }

  function addTrashIP(ip: string) { localAppData.value.trash_ip.add(ip), emit('mutateAppData', localAppData.value) }
  function removeTrashIP(ip: string) { localAppData.value.trash_ip.delete(ip), emit('mutateAppData', localAppData.value) }

  function addPinned(address: string) { localAppData.value.pinned.add(address), emit('mutateAppData', localAppData.value) }
  function removePinned(address: string) { localAppData.value.pinned.delete(address), emit('mutateAppData', localAppData.value) }

  function escapeButton() { lastSelectedServer.value = null, selectedServer.value = null }

  function displayTrashedServers() {
    emit('mutateConfig', { ...props.config, show_trashed_servers: true})
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
        } else {
          toList = 'pinned'
        }
      } else {
        toList = 'main'
      }
      await updateAppDataWithList(selectedServ, toList)

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

  async function updateAppDataWithList(server: Quake3Server, toList: string){
    if (toList == 'pinned' && !props.appData.custom.has(server.address)) {
      addPinned(server.address)
    }
    if (toList == 'trash' 
        && !props.appData.trash.has(server.address)
        && !props.appData.trash_ip.has(server.ip)
      ) {
      addTrash(server.address)
    }
    if (toList == 'main' && server.list == 'pinned') {
      removePinned(server.address)
    }
    if (toList == 'main' && server.list == 'trash') {
      removeTrash(server.address)
    }
  }

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
    emitComponentName() 
  }) 

  onDeactivated(() => {
    document.removeEventListener('keydown', handleKeyDown)
    document.removeEventListener('keyup', handleKeyUp)
  })

  onMounted(async () => {
    emitComponentName()    
    await refreshServers(true)
  })


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
          :isSelected="server === selectedServer && displayDetails"
          :refreshing="refreshingSingle"
          :altKeyHeld="altKeyHeld"
          :levelshotPath="levelHasLevelshot(server.map) ? levelshots![server.map.toLowerCase()] : null"
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
          :isSelected="server === selectedServer && displayDetails"
          :refreshing="refreshingSingle"
          :altKeyHeld="altKeyHeld"
          :displayDetailsOnMount="keepSelectedDetailsOpen"
          :levelshotPath="levelHasLevelshot(server.map) ? levelshots![server.map.toLowerCase()] : null"
          tabindex="0"
          @click="clickServer(server, index, $event);"
          @showDetails="displayDetails = true"
          @hideDetails="displayDetails = false"
          @contextmenu.prevent="rightClickToSelect(server); refreshSingleServer();"
          @detailsDisplayedOnUnmount="keepSelectedDetailsOpen = !keepSelectedDetailsOpen"
          />    
        <div v-if="showTrashedServers">
          <ServerRow v-for="(server, index) in trashServers" 
          class="row trash"
          :key="server.address" 
          :id="server === selectedServer ? 'selected' : `trash-${index}`"
          :server="server"
          :isSelected="server === selectedServer && displayDetails"
          :refreshing="refreshingSingle"
          :altKeyHeld="altKeyHeld"
          :levelshotPath="levelHasLevelshot(server.map) ? levelshots![server.map.toLowerCase()] : null"
          tabindex="0"
          @click="clickServer(server, index, $event);"
          @showDetails="displayDetails = true"
          @hideDetails="displayDetails = false"
          @contextmenu.prevent="rightClickToSelect(server); refreshSingleServer();"
          /> 
        </div> 
        <div v-if="!showTrashedServers && trashLength > 0" class="hidden-trash" @click="displayTrashedServers()">
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
      <span class="footer-data-right">Servers: {{ serverDetails.length }}</span>    
    </div>
    <div class="table-footer-left">  
      <img @mouseover="masterServerHover=true"
            @mouseleave="masterServerHover=false" 
            src="../assets/icons/q3-white.svg" 
            class="footer-icon"
            @click="showPopup='masterSettings'">
      <span class="footer-url">master.quake3arena.com</span>
      <div v-if="masterServerHover" class="master-servers">
        <div v-for="master in appData.masters" style="padding-right: 40px;">
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
        <div v-for="address in appData.custom"> 
          <button @click="removeCustom(address)" class="close-button"><img src="../assets/icons/x.svg" width="8px" /></button>
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
        <div v-for="ip in appData.trash_ip">
          <button @click="removeTrashIP(ip)" class="close-button"><img src="../assets/icons/x.svg" width="8px" /></button>
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
      
    <Modal v-if="showPopup=='masterSettings'" :popupType="'center'" @executeModal="handleMasterServerPopup(true)" @cancelModal="handleMasterServerPopup(true)">   
      <div v-for="master in localAppData.masters" style="height: 32px;">
        <span><input type="checkbox" v-model="master.active"></span>
        <text :style="master.unreachable ? 'color: #aaa; text-decoration: line-through;' : ''" class="ml-1">{{ master.game }}: {{ master.name }} </text>
      </div>
      <div style="margin: 0px 4px 0px 2px; text-align: left;">
        <text>+</text>
        <text class="ml-1">Quake 3 Master Protocol</text>
        <text class="ml-1"><input type="radio" :value="Number(68)" v-model="q3MasterProtocol">68</text>
        <text class="ml-1"><input type="radio" :value="Number(43)" v-model="q3MasterProtocol">43</text>
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

  .master-servers {
    padding: 14px;
    background-color: var(--alt-bg);
    color: #fff;
    position: absolute;
    bottom: 44px;
    left: 12px;
    border-radius: 0.2rem;
    font-size: 100%;
    max-width: max-content;
    max-height: max-content;
    white-space: nowrap;
    border: 1px solid var(--main-bg);
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