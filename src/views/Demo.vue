<script setup lang="ts">
  import DemoRow from '@/components/DemoRow.vue'
  import Loading from '@/components/Loading.vue'
  import { invoke } from '@tauri-apps/api/core'
  import { sep } from '@tauri-apps/api/path'
  import { info } from '@tauri-apps/plugin-log'
  import { ensureError } from '@/utils/util'
  import { type Config, type AppData } from '@/models/config'
  import { type Q3Executable } from '@/models/client'
  import { type Demo } from '@/models/demo'
  import { watch, nextTick, defineProps, defineEmits, ref, computed, onMounted, onActivated } from 'vue'
  import { useVirtualScroll } from '@/composables/virtualscroll'
  import { useClickRow } from '@/composables/clickrow'
  import { useLevelshot } from '@/composables/levelshot'

  const props = defineProps<{ config: Config, appData: AppData, showUnreachableServers: boolean, showTrashedServers: boolean, activeClient: Q3Executable | null }>()

  const emit = defineEmits<{
    mutateConfig: [Config]
    mutateAppData: [AppData]
    spawnQuake: [string[]]
    addQ3Client: [Q3Executable]
    emitComponentName: [string]
    errorAlert: [string]
    infoAlert: [string]
  }>()

  const componentName = ref('Demo Browser')

  async function pickClient() {
    try {
      let new_client: Q3Executable = await invoke('pick_client')

      if (new_client != null) {
        emit('addQ3Client', new_client)
      }
    } catch (err) {
      emit('errorAlert', ensureError(err).message)
    }
  }

  const loading = ref(false)
  const loadingEvent = ref('')

  const { levelshots, syncLevelshots, levelHasLevelshot } = useLevelshot()

  watch(() => props.activeClient, async(newVal, oldVal) => {
    if (newVal?.name != oldVal?.name) {
      await getDemos()
    }
  })

  const demos = ref<Demo[]>([])
  const demosLastRefresh = ref<Demo[]>([])
  const searchPaths = ref<string[]>([])

  async function getDemos() {
    if (props.activeClient == null || loading.value) {
      return
    }

    const startTime = performance.now()

    loading.value = true
    loadingEvent.value = 'parsing demos...'
    selectedDemo.value = null
    demosLastRefresh.value = []
    demos.value = []
    searchQuery.value = ''
    sortDesc.value = false
    currentSort.value = ''

    try {
      await syncLevelshots(props.activeClient, false)
      searchPaths.value = await invoke('get_client_paths', { activeClient: props.activeClient })

      demosLastRefresh.value = await invoke('get_demos_rayon', { searchPaths: searchPaths.value })
      demos.value = demosLastRefresh.value
    } catch (err) {
      emit('errorAlert', ensureError(err).message)
    }

    loading.value = false
    loadingEvent.value = ''
    handleScroll()

    const executionTime = performance.now() - startTime

    info(`${totalDemos.value} demos read in ${parseFloat((executionTime / 1000).toFixed(2))} seconds`)
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
        demosLastRefresh.value[i].file_name.toLowerCase().includes(query) ||
        demosLastRefresh.value[i].gamename.toLowerCase().includes(query)
      ) {
        filteredDemos.push(demosLastRefresh.value[i])
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

  function spawnQuake() {
    if (selectedDemo.value != null) {
      let relative_index = selectedDemo.value.path.indexOf(sep() + 'demos')
      let relative_path = selectedDemo.value.path.substring(relative_index + 7)
      let args = ['+set', 'fs_game', selectedDemo.value.gamename, '+demo', `\"${relative_path}\"`]
      emit('spawnQuake', args)
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

    selectedDemo.value = demos.value[selectedDemoIndex.value + increment]

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
    if (target.id == 'moreButton') { return }

    handleClick(clicked, target)

    if (dblClickHappenedOnSameObject.value) {
      spawnQuake()
      resetDblClickTimeout()
    }
  }

  const keepSelectedDetailsOpen = ref(false)

  onMounted(async () => {
    emit('emitComponentName', componentName.value)
    await getDemos()
  })

  onActivated(async () => {
    emit('emitComponentName', componentName.value)
  })
</script>

<template>
  <div class="table-header-base no-select">
    <div class="table-header-right">
      <input class="search" type="text" placeholder="search" v-model="searchQuery" />
    </div>
    <div class="table-header-left">
      <button class="connect-button" :disabled="selectedDemo == null" @click="spawnQuake()">Connect</button>
      <button class="refresh-button" @click="getDemos()">Refresh</button>
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
    <div v-if="!loading && props.activeClient" :style="{ height: virtualHeight + 'px' }">
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
          :levelshotPath="levelHasLevelshot(demo.mapname) ? levelshots![demo.mapname.toLowerCase()] : null"
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
    <div v-if="!props.activeClient">
      <div class="center"><button class="select-path-button" @click="pickClient()">Set a Quake 3 Client</button></div>
      <div v-for="(_, index) in 48" class="row" :style="index % 2 ? 'background-color: rgba(23, 32, 45, 0.3);' : ''"></div>
    </div>
  </div>

  <div class="table-footer">
    <div class="table-footer-right">
      <span class="footer-data-right" v-if="searchQuery.length == 0">Demos: {{ demosLastRefresh.length }}</span>
      <span class="footer-data-right" v-if="searchQuery.length > 0">Demos: {{ demos.length }}</span>
    </div>
    <div class="table-footer-left">
      <img src="../assets/icons/q3-white.svg" class="footer-icon" @click="pickClient()" />
      <span v-if="searchPaths.length" class="footer-url">{{ searchPaths[0] + sep() + 'demos' }}</span>
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

  .select-path-button {
    background-color: rgba(0, 0, 0, 0);
    color: white;
    border: 1px solid #00ffff;
    border-radius: 0.2rem;
    cursor: pointer;
    font-size: 160%;
    padding: 2px 10px 2px 10px;
    font-weight: 400;
  }

  .select-path-button:hover {
    background-color: var(--main-bg);
  }
</style>
