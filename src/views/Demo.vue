<script setup lang="ts">
  import DemoRow from '@/components/DemoRow.vue'
  import Loading from '@/components/Loading.vue'
  import { invoke } from '@tauri-apps/api/core'
  import { sep } from '@tauri-apps/api/path'
  import { info } from '@tauri-apps/plugin-log'
  import { ensureError } from '@/utils/util'
  import { type Demo } from '@/models/demo'
  import { type WatchHandle, watch, nextTick, defineEmits, ref, computed, onMounted, onActivated, onDeactivated } from 'vue'
  import { useVirtualScroll } from '@/composables/virtualscroll'
  import { useClickRow } from '@/composables/clickrow'
  import { useLevelshot } from '@/composables/levelshot'
  import { useConfig } from '@/composables/config'
  import { useClient } from '@/composables/client'

  const emit = defineEmits<{spawnQuake: [string[]], emitComponentName: [string], alert: [string, string]}>()
  defineProps<{ latestGithubVersion: string | null }>()

  const componentName = ref('Demo Browser')
  const { config } = useConfig()
  const { activeClient, activeClientPaths, pickClient } = useClient()

  onMounted(async () => {
    emit('emitComponentName', componentName.value)
    await getDemos(true)
  })

  let stopWatchingClient: WatchHandle; 
  const clientWhenDeactivated = ref(activeClient.value)

  onActivated(async () => {
    if (clientWhenDeactivated.value?.name != activeClient.value?.name || 
        clientWhenDeactivated.value?.gamename != activeClient.value?.gamename) 
    { 
      await getDemos(true) 
    }
    emit('emitComponentName', componentName.value)
    stopWatchingClient = watch(activeClient, async(newVal, oldVal) => {
      if (newVal?.name != oldVal?.name || newVal?.gamename != oldVal?.gamename) {
        await getDemos(true)
      }
    })
  });

  onDeactivated(async () => {
    clientWhenDeactivated.value = activeClient.value
    if (stopWatchingClient) {
      stopWatchingClient();
    }
  });

  async function pickQ3Client() {
    try {
      let isNewClient = await pickClient()
      if (!isNewClient) {
        emit('alert', 'info', 'client already added')
      }
    } catch (err) {
      emit('alert', 'error', ensureError(err).message)
    }
  }

  const loading = ref(false)
  const loadingEvent = ref('')

  const { levelshots } = useLevelshot()

  const demos = ref<Demo[]>([])
  const demosLastRefresh = ref<Demo[]>([])
  const demosCache = ref<Map<string, string>>(new Map)

  async function getDemos(fullRefresh: boolean) {
    if (activeClient.value == null || loading.value) {
      return
    }

    const startTime = performance.now()
    let num_got = 0

    loading.value = true
    loadingEvent.value = 'parsing demos...'
    selectedDemo.value = null

    if (fullRefresh) {
      demosLastRefresh.value = []
      demos.value = []
      demosCache.value.clear()
    }
    
    searchQuery.value = ''
    sortDesc.value = false
    currentSort.value = ''

    try {
      let new_demos: Demo[] = await invoke('get_demos', { 
                      searchPaths: activeClientPaths.value, 
                      cache: demosCache.value, 
                      allData: config.value.get_full_demo_data 
                    })
                    
      num_got = new_demos.length

      demosLastRefresh.value = demosLastRefresh.value.concat(new_demos)
      demos.value = demosLastRefresh.value

      demosCache.value.clear()
      demosLastRefresh.value.forEach((d) => {
        demosCache.value.set(d.path, 'exists')
      })

    } catch (err) {
      emit('alert', 'error', ensureError(err).message)
    }

    loading.value = false
    loadingEvent.value = ''
    handleScroll()
    const executionTime = performance.now() - startTime

    info(`${num_got} demos added in ${parseFloat((executionTime / 1000).toFixed(2))} seconds`)
  }

  const searchQuery = ref('')

  watch(searchQuery, async (newSearch, _oldSearch) => {
    selectedDemo.value = null
    lastSelectedDemo.value = null
    sortDesc.value = false
    currentSort.value = ''

    let query = newSearch.toLowerCase()

    let filteredDemos: Demo[] = []

    for (let i = 0; i < demosLastRefresh.value.length; i++) {
      if (
        demosLastRefresh.value[i]!.file_name.toLowerCase().includes(query) ||
        demosLastRefresh.value[i]!.gamename.toLowerCase().includes(query)
      ) {
        filteredDemos.push(demosLastRefresh.value[i]!)
      }
    }
    demos.value = filteredDemos
  })

  const sortDesc = ref(false)
  const currentSort = ref('')

  function getArrowSort(column: string) {
    if (currentSort.value != column) {
      return 'sort-arrow-default'
    }
    if (sortDesc.value && currentSort.value == column) {
      return 'sort-arrow-desc'
    }
    if (!sortDesc.value && currentSort.value == column) {
      return 'sort-arrow-asc'
    }
  }

  function sortDemos(column: string) {
    if (currentSort.value == column || currentSort.value == '') {
      sortDesc.value = !sortDesc.value
    } else {
      sortDesc.value = true
    }

    selectedDemo.value = null
    currentSort.value = column

    if (column == 'name') {
      demos.value.sort((a, b) => {
        if (a['player_pov'][column] > b['player_pov'][column]) {
          return sortDesc.value ? -1 : 1
        }
        if (a['player_pov'][column] < b['player_pov'][column]) {
          return sortDesc.value ? 1 : -1
        }
        return 0
      })
    } else if (column == 'file_name' || column == 'duration' || column == 'gamename' || column == 'mapname' || column == 'g_gametype') {
      demos.value.sort((a, b) => {
        if (a[column] > b[column]) {
          return sortDesc.value ? -1 : 1
        }
        if (a[column] < b[column]) {
          return sortDesc.value ? 1 : -1
        }
        return 0
      })
    }
  }

  const selectedDemo = ref<Demo | null>(null)
  const lastSelectedDemo = ref<Demo | null>(null)

  function getDemoIndex(d: Demo) {
    return demos.value.indexOf(d)
  }

  watch(selectedDemo, (_newSelectedDemo, _old) => {
    keepSelectedDetailsOpen.value = false
  })

  function escapeButton() {
    lastSelectedDemo.value = null
    selectedDemo.value = null
  }

  async function spawnQuake() {
    if (selectedDemo.value == null) { return }

    try {
      let args = ['+set', 'fs_game', selectedDemo.value.gamename, '+exec', 'sarge-launcher-demo.cfg']
      let demos_index = selectedDemo.value.path.indexOf(sep() + 'demos')
      let relative_path = selectedDemo.value.path.substring(demos_index + 7)
      let path = relative_path.substring(0, relative_path.lastIndexOf('.'))
  
      await invoke('create_demo_script', {
        activeClient: activeClient.value, 
        fsGame: selectedDemo.value.gamename, 
        demoPath: path, 
        close: config.value.autoclose_demo, 
        loopD: config.value.loop_demo
      })      
      emit('spawnQuake', args)
    } catch (err) {
      emit('alert', 'error', ensureError(err).message)
    }   
  }
  
  const totalDemos = computed(() => {
    return demos.value.length
  })

  const { 
    translateY,
    virtualHeight,
    marginTop,
    virtualStartIndex,
    virtualEndIndex,
    handleScroll
  } = useVirtualScroll('demoTable', totalDemos);

  const getVirtualRows = computed(() => {
    return demos.value.slice(virtualStartIndex.value, virtualEndIndex.value)
  })

  const selectedDemoIndex = computed(() => {
    if (selectedDemo.value) {
      return demos.value.indexOf(selectedDemo.value)
    }
    return -1
  })

  function keySelect(increment: number) {
    if (selectedDemo.value == null || (selectedDemoIndex.value + increment) < 0 || (selectedDemoIndex.value + increment) == totalDemos.value) {
      return
    }

    selectedDemo.value = demos.value[selectedDemoIndex.value + increment]!

    nextTick(() => {
      // the real dom is now updated (document.)
      let selected = document.getElementById('selected')

      if (selected) {
        selected?.scrollIntoView({ behavior: 'instant' as ScrollBehavior, block: 'start', inline: 'nearest' })
        selected?.focus()
      }
    })
  }

  const displayDetails = ref(false)

  const {
   handleClick,
   dblClickHappenedOnSameObject,
   resetDblClickTimeout,
   rightClickToSelect
  } = useClickRow(selectedDemo, lastSelectedDemo, displayDetails);

  function clickDemo(clicked: Demo, event: MouseEvent) {
    let target = event.target as HTMLTextAreaElement;
    if (target.id == 'moreButton' || target.id == 'levelshot') { return }

    handleClick(clicked, target)

    if (dblClickHappenedOnSameObject.value) {
      spawnQuake()
      resetDblClickTimeout()
    }
  }

  const keepSelectedDetailsOpen = ref(false)
  const showSearchPaths = ref(false)
  
</script>

<template>
  <div class="table-header-base no-select">
    <div class="table-header-right">
      <input class="search" type="text" placeholder="search" v-model="searchQuery" />
    </div>
    <div class="table-header-left">
      <button class="connect-button" :disabled="selectedDemo == null" @click="spawnQuake()">Connect</button>
      <button class="refresh-button" @click="getDemos(false)">Refresh</button>
    </div>

    <div class="table-column-header">
      <span style="width: 1%"></span>
      <span style="width: 9%; text-align: left"><span class="sort-header" @click="sortDemos('gamename')">game</span><span :class="getArrowSort('gamename')" @click="sortDemos('gamename')" /></span>
      <span style="width: 8%; text-align: left"><span class="sort-header" @click="sortDemos('g_gametype')">type</span><span :class="getArrowSort('g_gametype')" @click="sortDemos('g_gametype')" /></span>
      <span style="width: 12%; text-align: left"><span class="sort-header" @click="sortDemos('name')">pov</span><span :class="getArrowSort('name')" @click="sortDemos('name')" /></span>
      <span style="width: 44%; text-align: left"><span class="sort-header" @click="sortDemos('file_name')">demo</span><span :class="getArrowSort('file_name')" @click="sortDemos('file_name')" /></span>
      <span style="width: 1%"></span>
      <span style="width: 14%; text-align: left"><span class="sort-header" @click="sortDemos('mapname')">map</span><span :class="getArrowSort('mapname')" @click="sortDemos('mapname')" /></span>
      <span style="width: 1%"></span>
      <span style="width: 12%; text-align: left"><span class="sort-header" @click="sortDemos('duration')">duration</span><span :class="getArrowSort('duration')" @click="sortDemos('duration')" /></span>
      <span style="width: 2%"></span>
    </div>
  </div>

  <div
    class="scrollable-container"
    @keydown.up.prevent="keySelect(-1)"
    @keydown.down.prevent="keySelect(1)"
    @keydown.enter.prevent="spawnQuake()"
    @keydown.esc.prevent="escapeButton()"
    ref="demoTable"
    id="demoTable"
  >
    <div v-if="!loading && activeClient" :style="{ height: virtualHeight + 'px' }">
      <div
        class="main"
        :style="{ transform: 'translateY(' + translateY + 'px)', marginTop: marginTop + 'px' }"
      >
        <DemoRow
          v-for="demo in getVirtualRows"
          class="row"
          :style="getDemoIndex(demo) % 2 ? 'background-color: rgba(23, 32, 45, 0.3);' : ''"
          :key="demo.path"
          :id="demo === selectedDemo ? 'selected' : `demo-${getDemoIndex(demo)}`"
          :demo="demo"
          :isSelected="demo === selectedDemo && displayDetails"
          :displayDetailsOnMount="keepSelectedDetailsOpen"
          :levelshotPath="levelshots[demo.mapname.toLowerCase()] ?? null"
          tabindex="0"
          @click="clickDemo(demo, $event)"
          @contextmenu.prevent="rightClickToSelect(demo)"
          @showDetails="displayDetails = true"
          @hideDetails="displayDetails = false"
          @detailsDisplayedOnUnmount="keepSelectedDetailsOpen = !keepSelectedDetailsOpen"
        />
      </div>
    </div>
    <div v-if="loading">
      <Loading :position="'center'" :message="loadingEvent" :size="90" />
      <div v-for="(_, index) in 48" class="row" :style="index % 2 ? 'background-color: rgba(23, 32, 45, 0.3);' : ''"></div>
    </div>
    <div v-if="!activeClient">
      <div class="center"><button class="select-path-button" @click="pickQ3Client()">Set a Quake 3 Client</button></div>
      <div v-for="(_, index) in 48" class="row" :style="index % 2 ? 'background-color: rgba(23, 32, 45, 0.3);' : ''"></div>
    </div>
  </div>

  <div class="table-footer">
    <div class="table-footer-right">
      <span class="footer-data-right" v-if="searchQuery.length == 0">Demos: {{ demosLastRefresh.length }}</span>
      <span class="footer-data-right" v-if="searchQuery.length > 0">Demos: {{ demos.length }}</span>
    </div>
    <div class="table-footer-left">
      <button v-if="activeClient" 
            @mouseover="showSearchPaths=true"
            @mouseleave="showSearchPaths=false" 
            class="search-paths">
        Search Paths
      </button>
      <div v-if="activeClientPaths && showSearchPaths" class="footer-popup">
        <div v-for="p in activeClientPaths" style="padding-right: 40px;">
          <div style="display: inline-block; width: 15%;">{{ p }} </div>
        </div>
      </div> 
    </div>
  </div>
</template>

<style scoped>
  .main {
    min-height: 24px;
    position: relative;
    cursor: default;
  }

  .center {
    position: absolute;
    left: 50%;
    top: 50%;
    transform: translate(-50%, -50%);
    color: white;
  }
</style>
