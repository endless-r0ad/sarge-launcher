<script setup lang="ts">
  import Loading from '@/components/Loading.vue'
  import { invoke } from '@tauri-apps/api/core'
  import { info } from '@tauri-apps/plugin-log'
  import { ensureError } from '@/utils/util'
  import { type Config, type AppData } from '@/models/config'
  import { type Q3Executable } from '@/models/client'
  import { type Level } from '@/models/level'
  import { useVirtualScroll } from '@/composables/virtualscroll'
  import { useClickRow } from '@/composables/clickrow'
  import { useLevelshot } from '@/composables/levelshot'
  import { watch, nextTick, defineProps, defineEmits, ref, computed, onMounted, onActivated } from 'vue'
  import { type Bot } from '@/models/singleplayer'
  import { Q3_BOT_NAMES } from '@/utils/util'

  const props = defineProps<{ config: Config; appData: AppData; showUnreachableServers: boolean; showTrashedServers: boolean, activeClient: Q3Executable | null }>()

  const emit = defineEmits<{
    mutateConfig: [Config]
    mutateAppData: [AppData]
    spawnQuake: [string[]]
    addQ3Client: [Q3Executable]
    emitComponentName: [string]
    errorAlert: [string]
    infoAlert: [string]
  }>()

  const componentName = ref('Single Player')

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

  watch(() => props.activeClient, async(newVal, oldVal) => {
    if (newVal?.name != oldVal?.name) {
      await getLevels()
    }
  })


  const bots_team_free = ref<Bot[]>([])
  const bots_team_red = ref<Bot[]>([])
  const bots_team_blue = ref<Bot[]>([])

  function pushBot(bot: Bot) {
    if (bot.team == 'Free' && bots_team_free.value.length < Math.min(sv_maxclients.value-1, 15)) {
        bots_team_free.value.push(bot)
    }
    if (bot.team == 'Red' && bots_team_red.value.length < (sv_maxclients.value - bots_team_blue.value.length - 1)) {
        bots_team_red.value.push(bot)
    }
    if (bot.team == 'Blue' && bots_team_blue.value.length < (sv_maxclients.value - bots_team_red.value.length - 1)) {
        bots_team_blue.value.push(bot)
    }
  }

  const loading = ref(false)
  const loadingEvent = ref('')

  const { levels, levelshots, syncLevelshots, levelHasLevelshot } = useLevelshot()

  const levelsLastRefresh = ref<Level[]>([])
  const showBaseLevelsOnly = ref(false)

  watch(showBaseLevelsOnly, (newVal, _oldVal) => {
    if (newVal) {
      if (props.activeClient && props.activeClient.gamename) {
        switch (props.activeClient.gamename) {
          case "baseq3":
            levels.value = levelsLastRefresh.value.filter((m) => m.pk3_name == 'pak0')
            break
          case "baseoa":
            levels.value = levelsLastRefresh.value.filter((m) => m.pk3_name == 'pak1-maps' || m.pk3_name == 'pak6-patch085' || m.pk3_name == 'pak6-patch088')
            break
          case "cpma":
            levels.value = levelsLastRefresh.value.filter((m) => m.pk3_name.includes('map_cpm'))
            break
          case "defrag":
            levels.value = levelsLastRefresh.value.filter((m) => m.is_defrag)
            break
          case "q3ut4":
            levels.value = levelsLastRefresh.value.filter((m) => m.level_name.includes('ut4'))
            break
          default:
            levels.value = levelsLastRefresh.value.filter((m) => m.pk3_name == 'pak0')
            break
        }
      }
    } else {
      levels.value = levelsLastRefresh.value
    }
    selectedLevel.value = null
  })

  async function getLevels() {
    if (props.activeClient == null || loading.value) {
      return
    }

    const startTime = performance.now()

    loading.value = true
    loadingEvent.value = 'getting levels...'
    selectedLevel.value = null
    levelsLastRefresh.value = []
    searchQuery.value = ''
    sortDesc.value = false
    currentSort.value = ''

    try {
      await syncLevelshots(props.activeClient, true)
      levelsLastRefresh.value = levels.value
    } catch (err) {
      emit('errorAlert', ensureError(err).message)
    }

    console.log('levels are ', levels.value)
    showBaseLevelsOnly.value = false
    loading.value = false
    loadingEvent.value = ''
    handleScroll()

    const executionTime = performance.now() - startTime

    info(`${levelsLength.value} levels read in ${parseFloat((executionTime / 1000).toFixed(2))} seconds`)
  }

  const searchQuery = ref('')

  watch(searchQuery, async (newSearch, _oldSearch) => {
    selectedLevel.value = null
    lastSelectedLevel.value = null
    sortDesc.value = false
    currentSort.value = ''

    let query = newSearch.toLowerCase()

    let filteredMaps: Level[] = []

    for (let i = 0; i < levelsLastRefresh.value.length; i++) {
      if (levelsLastRefresh.value[i].level_name.toLowerCase().includes(query) || 
          levelsLastRefresh.value[i].author.toLowerCase().includes(query)) {
        filteredMaps.push(levelsLastRefresh.value[i])
      }
    }
    levels.value = filteredMaps
  })

  const sortDesc = ref(false)
  const currentSort = ref('')

  function getArrowSort(column: string) {
    if (currentSort.value != column) { return 'sort-arrow-default'}
    if (sortDesc.value && currentSort.value == column) { return 'sort-arrow-desc'}
    if (!sortDesc.value && currentSort.value == column) { return 'sort-arrow-asc'}
  }

  function sortMaps(column: string){
    if (currentSort.value == column || currentSort.value == ''){
      sortDesc.value = !sortDesc.value
    } else {
      sortDesc.value = true;
    }

    selectedLevel.value = null
    currentSort.value = column

    if (column == 'level_name' ) {
      levels.value.sort((a, b) => {
        if (a[column].toLowerCase() > b[column].toLowerCase()) {
          return ( sortDesc.value ? -1 : 1 );
        }
        if (a[column].toLowerCase() < b[column].toLowerCase()) {
          return ( sortDesc.value ? 1 : -1 );
        }
        return 0;
      });
    }
  }

  const selectedLevel = ref<Level | null>(null)
  const lastSelectedLevel = ref<Level | null>(null)

  const gameType = ref<number | null>(2)
  const teamSelect = ref<'Free' | 'Red' | 'Blue'>('Free')
  const difficulty = ref<1 | 2 | 3 | 4 | 5>(5)
  const cheats = ref(false)
  const sv_maxclients = ref(8)

  const selectedMapIndex = computed(() => {
    if (selectedLevel.value) {
      return levels.value.indexOf(selectedLevel.value)
    }
    return -1
  })

  function getLevelIndex(d: Level) {
    return levels.value.indexOf(d)
  }

  function escapeButton() {
    lastSelectedLevel.value = null
    selectedLevel.value = null
  }

  function spawnQuake() {
    if (selectedLevel.value && gameType.value != null) {
      let launch = ''

      if (gameType.value == 2) {
        launch = cheats.value ? '+spdevmap' : '+spmap'
      } else if (gameType.value == 98) {
        launch = cheats.value ? '+devvq3' : '+vq3'
      } else if (gameType.value == 99) {
        launch = cheats.value ? '+devcpm' : '+cpm'
      } else {
        launch = cheats.value ? '+devmap' : '+map'
      }

      let args = [launch, selectedLevel.value.level_name, '+set', 'g_gametype', gameType.value.toString()]

      args.push(...['+set', 'sv_maxclients', sv_maxclients.value.toString()])
      args.push(...['+set', gameType.value == 2 ? 'g_spskill' : 'skill', difficulty.value.toString(), '+wait', '3'])

      if (gameType.value != 3 && gameType.value != 4 && props.activeClient?.gamename != 'defrag') {
        bots_team_free.value.forEach((bot) => {
          args.push(...['+addbot', bot.name, bot.difficulty.toString()])
        })
      }

      if (gameType.value == 3 || (gameType.value == 4 && props.activeClient?.gamename != 'defrag')) {
        bots_team_red.value.forEach((bot) => {
          args.push(...['+addbot', bot.name, bot.difficulty.toString(), bot.team])
        })

        bots_team_blue.value.forEach((bot) => {
          args.push(...['+addbot', bot.name, bot.difficulty.toString(), bot.team])
        })
      }

      args.push(...['+wait', '5'])
      args.push(...['+team', teamSelect.value])

      emit('spawnQuake', args)
    }
  }

  const getVirtualRows = computed(() => {
    return levels.value.slice(virtualStartIndex.value, virtualEndIndex.value)
  })

  function keySelect(increment: number) {
    if (selectedLevel.value == null || selectedMapIndex.value + increment < 0 || selectedMapIndex.value + increment == levelsLength.value) {
      return
    }

    selectedLevel.value = levels.value[selectedMapIndex.value + increment]

    nextTick(() => {
      // the real dom is now updated (document.)
      let selected = document.getElementById('selected')

      if (selected) {
        selected.scrollIntoView({ behavior: 'instant' as ScrollBehavior, block: 'start', inline: 'nearest' })
        selected.focus()
      }
    })
  }

  const { 
    handleClick, 
    dblClickHappenedOnSameObject, 
    resetDblClickTimeout, 
    rightClickToSelect 
  } = useClickRow(selectedLevel, lastSelectedLevel)

  function clickLevel(clicked: Level) {
    handleClick(clicked)

    if (dblClickHappenedOnSameObject.value) {
      spawnQuake()
      resetDblClickTimeout()
    }
  }

  const levelsLength = computed(() => {
    return levels.value.length
  })

  const { setScroller, translateY, virtualHeight, marginTop, virtualStartIndex, virtualEndIndex, handleScroll } = useVirtualScroll(
    'levelTable',
    levelsLength
  )

  setScroller({ rowHeight: 96, overscan: 8 })

  onMounted(async () => {
    emit('emitComponentName', componentName.value)
    await getLevels()
  })

  onActivated(async () => {
    emit('emitComponentName', componentName.value)
  })
</script>

<template>
  <div class="table-header-base no-select" style="height: 40px">
    <div class="table-header-right">
      <button class="refresh-button" :class="{ 'base-only': showBaseLevelsOnly }" @click="showBaseLevelsOnly = !showBaseLevelsOnly">
        {{ props.activeClient?.gamename }}
      </button>
      <input class="search" type="text" placeholder="search" v-model="searchQuery" />
    </div>
    <div class="table-header-left">
      <button class="connect-button" :disabled="!selectedLevel || gameType == null" @click="spawnQuake()">Connect</button>
      <button class="refresh-button" @click="getLevels()">Refresh</button>
      <span style="margin-left: 24px; text-align: left; color: #fff;">
        <span class="sort-header" @click="sortMaps('level_name')">map</span>
        <span :class="getArrowSort('level_name')" @click="sortMaps('level_name')" />
      </span>

    </div>
  </div>

  <div
    class="scrollable-container single-player no-select"
    @keydown.up.prevent="keySelect(-1)"
    @keydown.down.prevent="keySelect(1)"
    @keydown.enter.prevent="spawnQuake()"
    @keydown.esc.prevent="escapeButton()"
    ref="levelTable"
    id="levelTable"
  >
    <div v-if="!loading && props.activeClient" :style="{ height: virtualHeight + 'px' }">
      <div class="main" :style="{ transform: 'translateY(' + translateY + 'px)', marginTop: marginTop + 'px' }">
        <div
          v-for="level in getVirtualRows"
          class="level-row"
          :style="getLevelIndex(level) % 2 ? 'background-color: rgba(23, 32, 45, 0.3);' : ''"
          :key="level.path + level.pk3_name + level.level_name"
          :id="level === selectedLevel ? 'selected' : `level-${getLevelIndex(level)}`"
          tabindex="0"
          @click="clickLevel(level)"
          @contextmenu.prevent="rightClickToSelect(level)"
        >
          <div class="map-row">
            <img v-if="levelHasLevelshot(level.level_name)" class="map-img" :src="levelshots![level.level_name.toLowerCase()]"/>
            <img v-else class="map-img" src="../assets/icons/q3-white.svg" />
            <div style="width: 30%; text-align: left; white-space: nowrap; overflow: hidden; margin-left: 24px">
              <h3 style="margin: 8px 0 0 0;">{{ level.level_name }}</h3>
              <h6 v-if="level.long_name" v-html="level.long_name" style="font-style: italic; margin: 4px 0 0 0;" />
              <h6 v-if="level.year_created" style="font-style: italic; margin: 4px 0 0 0;" >{{ level.year_created }}</h6>
              <h6 v-if="level.author" v-html="level.author" style="margin: 12px 0 0 0;"/>     
            </div>
            <div style="margin-left: -184px; margin-top: 64px;">
              <span class="gametype-tag" v-for="l in level.gametype">{{ l }} </span>
            </div>           
          </div>
        </div>
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
      <span class="footer-data-right" v-if="searchQuery.length == 0 && !showBaseLevelsOnly">Maps: {{ levelsLastRefresh.length }}</span>
      <span class="footer-data-right" v-if="searchQuery.length > 0 || showBaseLevelsOnly">Maps: {{ levels.length }}</span>
    </div>
    <div class="table-footer-left">
      <img src="../assets/icons/q3-white.svg" class="footer-icon" @click="pickClient()" />
      <span class="footer-url">{{ props.activeClient?.exe_path }}</span>
    </div>
  </div>

  <div v-if="selectedLevel" class="game-setup no-select">
    <h2 style="text-align: center">Game Setup</h2>
    <div style="text-align: center; margin-bottom: 10px">
      <button v-if="props.activeClient?.gamename != 'defrag'" class="setup-button" :class="{ active: gameType == 2 }" @click="gameType = 2">SP</button>
      <button v-if="props.activeClient?.gamename != 'defrag'" class="setup-button" :class="{ active: gameType == 1 }" @click="gameType = 1">1v1</button>
      <button v-if="props.activeClient?.gamename != 'defrag'" class="setup-button" :class="{ active: gameType == 0 }" @click="gameType = 0">FFA</button>
      <button
        v-if="props.activeClient?.gamename != 'defrag'"
        class="setup-button"
        :class="{ active: gameType == 3 }"
        @click="
          gameType = 3;
          teamSelect = teamSelect == 'Free' ? 'Red' : teamSelect;
        "
      >
        TDM
      </button>
      <button
        v-if="props.activeClient?.gamename != 'defrag'"
        class="setup-button"
        :class="{ active: gameType == 4 }"
        @click="
          gameType = 4;
          teamSelect = teamSelect == 'Free' ? 'Red' : teamSelect;
        "
      >
        CTF
      </button>

      <button v-if="props.activeClient?.gamename == 'defrag'" class="setup-button" :class="{ active: gameType == 98 }" @click="gameType = 98">VQ3</button>
      <button v-if="props.activeClient?.gamename == 'defrag'" class="setup-button" :class="{ active: gameType == 99 }" @click="gameType = 99">CPM</button>
    </div>
    <div style="text-align: center; margin-bottom: 10px">
      <button v-if="props.activeClient?.gamename != 'defrag'" class="dif-button" :class="{ active: difficulty == 1 }" @click="difficulty = 1">
        i can win
      </button>
      <button v-if="props.activeClient?.gamename != 'defrag'" class="dif-button" :class="{ active: difficulty == 2 }" @click="difficulty = 2">
        bring it on
      </button>
      <button v-if="props.activeClient?.gamename != 'defrag'" class="dif-button" :class="{ active: difficulty == 3 }" @click="difficulty = 3">
        hurt me plenty
      </button>
      <button v-if="props.activeClient?.gamename != 'defrag'" class="dif-button" :class="{ active: difficulty == 4 }" @click="difficulty = 4">
        hardcore
      </button>
      <button v-if="props.activeClient?.gamename != 'defrag'" class="dif-button" :class="{ active: difficulty == 5 }" @click="difficulty = 5">
        nightmare!
      </button>
    </div>

    <div style="display: flex; height: 40%; user-select: none">
      <img
        v-if="levelHasLevelshot(selectedLevel.level_name)"
        style="width: 50%"
        :src="levelshots![selectedLevel.level_name.toLowerCase()]"
      />
      <img v-else style="width: 50%" src="../assets/icons/q3-white.svg" />
      <div style="width: 50%; text-align: center; overflow: hidden scroll; padding: 4px 0px 4px 4px" v-if="gameType != 4 && gameType != 3">
        <button class="bot-button" style="margin: 0px 0px 6px 30px; color: #fff;">you</button>
        <div v-for="b in bots_team_free" style="white-space: nowrap;">
          <button @click="bots_team_free.splice(bots_team_free.indexOf(b), 1)" class="close-button">
            <img src="../assets/icons/x.svg" width="8px" />
          </button>
          <label class="bot-button" @click="b.name = Q3_BOT_NAMES[(Q3_BOT_NAMES.indexOf(b.name) + 1) % 23]">{{ b.name }}</label>
          <label class="bot-button" style="margin-left: 6px" @click="b.difficulty = (b.difficulty % 5) + 1">{{ b.difficulty }}</label>
        </div>
      </div>
      <div v-if="gameType == 3 || gameType == 4" class="team red">
        <button v-if="teamSelect == 'Red'" class="bot-button" style="margin: 0px 0px 6px 30px; color: #fff;">you</button>
        <div v-for="b in bots_team_red" style="white-space: nowrap;">
          <button @click="bots_team_red.splice(bots_team_red.indexOf(b), 1)" class="close-button">
            <img src="../assets/icons/x.svg" width="8px" />
          </button>
          <label class="bot-button" @click="b.name = Q3_BOT_NAMES[(Q3_BOT_NAMES.indexOf(b.name) + 1) % 23]">{{ b.name }}</label>
          <label class="bot-button" style="margin-left: 6px" @click="b.difficulty = (b.difficulty % 5) + 1">{{ b.difficulty }}</label>
        </div>
      </div>
      <div v-if="gameType == 3 || gameType == 4" class="team blue">
        <button v-if="teamSelect == 'Blue'" class="bot-button" style="margin: 0px 0px 6px 30px; color: #fff;">you</button>
        <div v-for="b in bots_team_blue" style="white-space: nowrap;">
          <button @click="bots_team_blue.splice(bots_team_blue.indexOf(b), 1)" class="close-button">
            <img src="../assets/icons/x.svg" width="8px" />
          </button>
          <label class="bot-button" @click="b.name = Q3_BOT_NAMES[(Q3_BOT_NAMES.indexOf(b.name) + 1) % 23]">{{ b.name }}</label>
          <label class="bot-button" style="margin-left: 6px" @click="b.difficulty = (b.difficulty % 5) + 1">{{ b.difficulty }}</label>
        </div>
      </div>
    </div>

    <div style="display: flex; margin: 10px 0">
      <div style="width: 50%; text-align: center">
        <button class="dif-button" @click="sv_maxclients = (sv_maxclients % 24) + 1">sv_maxclients: {{ sv_maxclients }}</button>
        <button
          v-if="gameType == 4 || gameType == 3"
          class="dif-button"
          :style="teamSelect == 'Red' ? 'background-color: rgba(255, 0, 0, 0.2)' : 'background-color: rgba(0, 0, 255, 0.2);'"
          @click="teamSelect = teamSelect == 'Red' ? 'Blue' : 'Red'"
        >
          Team: {{ teamSelect }}
        </button>
      </div>
      <div style="width: 50%; text-align: center" v-if="gameType != 4 && gameType != 3">
        <button
          v-if="gameType != 4 && gameType != 3"
          class="dif-button"
          style="display: inline-block"
          @click="pushBot({ name: 'sarge', difficulty: difficulty, team: 'Free' })"
        >
          +bot
        </button>
      </div>
      <div style="width: 25%; text-align: center" v-if="gameType == 4 || gameType == 3">
        <button
          v-if="gameType == 4 || gameType == 3"
          class="dif-button"
          style="display: inline-block"
          @click="pushBot({ name: 'sarge', difficulty: difficulty, team: 'Red' })"
        >
          +bot
        </button>
      </div>
      <div style="width: 25%; text-align: center" v-if="gameType == 4 || gameType == 3">
        <button
          v-if="gameType == 4 || gameType == 3"
          class="dif-button"
          style="display: inline-block"
          @click="pushBot({ name: 'sarge', difficulty: difficulty, team: 'Blue' })"
        >
          +bot
        </button>
      </div>
    </div>
    <button class="dif-button" :class="{ active: cheats }" @click="cheats = !cheats" style="margin-left: 5%;">cheats</button>

    <div style="text-align: center">
      <button class="setup-button" :disabled="!selectedLevel || gameType == null" @click="spawnQuake()">Connect</button>
    </div>
  </div>
</template>

<style scoped>
  .base-only-button {
    cursor: pointer;
    padding: 4px 15px;
    margin: 0px 4px;
    background: url('../assets/icons/q3-white.svg') center center no-repeat;
    background-size: 78%;
  }

  .base-only-button:hover {
    background-color: var(--main-bg);
    border-radius: 0.2rem;
  }

  .base-only {
    background-color: rgba(0, 143, 168, 0.514);
    border-radius: 0.2rem;
    cursor: pointer;
  }

  .single-player {
    height: calc(100% - 72px);
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

  .bot-button {
    background: rgba(0, 0, 0, 0);
    border: 1px solid var(--main-bg);
    border-radius: 0.2rem;
    padding: 3px 5px;
    font-size: 80%;
  }

  .bot-button:hover {
    background-color: var(--main-bg);
    cursor: pointer;
  }

  .setup-button {
    background-color: rgba(0, 0, 0, 0);
    color: white;
    border: 1px solid var(--main-bg);
    border-radius: 0.2rem;
    cursor: pointer;
    font-size: 150%;
    padding: 2px 10px 2px 10px;
    font-weight: 400;
    margin-right: 4px;
  }

  .setup-button:disabled,
  .setup-button[disabled] {
    background-color: rgba(0, 0, 0, 0);
    color: var(--main-bg);
    border: 1px solid var(--main-bg);
    border-radius: 0.2rem;
    cursor: default;
    font-size: 110%;
    padding: 2px 10px 2px 10px;
    font-weight: 400;
    margin-right: 4px;
  }

  .setup-button:hover:enabled {
    background-color: var(--main-bg);
  }

  .dif-button {
    background-color: rgba(0, 0, 0, 0);
    color: white;
    border: 1px solid var(--main-bg);
    border-radius: 0.2rem;
    cursor: pointer;
    font-size: 100%;
    padding: 2px 10px 2px 10px;
    font-weight: 400;
    margin-right: 4px;
  }

  .dif-button:disabled,
  .dif-button[disabled] {
    background-color: rgba(0, 0, 0, 0);
    color: var(--main-bg);
    border: 1px solid var(--main-bg);
    border-radius: 0.2rem;
    cursor: default;
    font-size: 100%;
    padding: 2px 10px 2px 10px;
    font-weight: 400;
    margin-right: 4px;
  }

  .dif-button:hover:enabled {
    background-color: var(--main-bg);
  }

  .game-setup {
    position: absolute;
    top: 96px;
    right: 52px;
    bottom: 82px;
    width: 50%;
    background-color: rgba(0, 0, 0, 0.15);
    border: 1px solid #00ffff;
    backdrop-filter: blur(10px);
    color: #fff;
  }

  .main {
    min-height: 96px;
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

  .map-row {
    color: white;
    display: flex;
    flex-direction: row;
    height: min-content;
    overflow: hidden;
  }

  .map-img {
    height: 96px;
    width: 128px;
  }

  .level-row {
    min-height: 96;
    color: white;
    overflow: hidden;
    outline: none;
  }

  .active {
    background-color: rgba(0, 143, 168, 0.514);
  }

  .team {
    width: 25%; 
    text-align: left; 
    overflow: hidden scroll; 
    padding: 4px 0px 4px 4px;
  }

  .red {
    background-color: rgba(255, 0, 0, 0.2); 
  }

  .blue {
    background-color: rgba(0, 0, 255, 0.2); 
  }
</style>
