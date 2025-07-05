<script setup lang="ts">
  import Loading from "@/components/Loading.vue"
  import { invoke } from "@tauri-apps/api/core"
  import { convertFileSrc } from "@tauri-apps/api/core"
  import { info } from "@tauri-apps/plugin-log"
  import { type Nullable, ensureError } from "@/utils/util"
  import { type Config, type AppData } from "@/models/config"
  import { type Q3Executable } from "@/models/client"
  import { type Level } from "@/models/level"
  import { useVirtualScroll } from '@/composables/virtualscroll';
  import { useClickRow } from '@/composables/clickrow';
  import { watch, nextTick, defineProps, defineEmits, ref, computed, onMounted, onActivated } from "vue"
  import { type Bot } from "@/models/singleplayer"
  import { Q3_BOT_NAMES, setDefaultImage } from "@/utils/util"

  const props = defineProps<{ config: Config, appData: AppData, showUnreachableServers: boolean, showTrashedServers: boolean }>()

  const emit = defineEmits<{
    mutateConfig: [Config]
    mutateAppData: [AppData]
    spawnQuake: [string]
    addQ3Client: [Q3Executable]
    emitConnectArgs: [string[]]
    emitComponentName: [string]
    errorAlert: [string]
    infoAlert: [string]
  }>()

  const componentName = ref("Single Player")
  const fsHomepath = ref(props.config.fs_homepath)

  async function pickFsHomepath() {
    let path: Nullable<string> = await invoke("pick_fs_homepath")

    if (path != null) {
      fsHomepath.value = path
      emit("mutateConfig", { ...props.config, fs_homepath: path })
      await getLevels()
    }
  }

  const bots_team_free = ref<Bot[]>([])
  const bots_team_red = ref<Bot[]>([])
  const bots_team_blue = ref<Bot[]>([])

  const loading = ref(false)
  const loadingEvent = ref("")

  const levels = ref<Level[]>([])
  const levelsLastRefresh = ref<Level[]>([])
  const showPak0LevelsOnly = ref(false)

  function pak0LevelsOnly() {
    if (showPak0LevelsOnly.value) {
      levels.value = levelsLastRefresh.value
      showPak0LevelsOnly.value = false
      selectedLevel.value = null
    } else {
      levels.value = levelsLastRefresh.value.filter((m) => m.pk3_name == 'pak0')
      showPak0LevelsOnly.value = true
      selectedLevel.value = null
    }
  }

  async function getLevels() {
    if (fsHomepath.value == null || loading.value) {
      return
    }

    const startTime = performance.now()

    loading.value = true
    loadingEvent.value = "getting levels..."
    selectedLevel.value = null
    levelsLastRefresh.value = []
    levels.value = []
    searchQuery.value = ""
    sortDesc.value = false
    currentSort.value = ""

    try {
      levelsLastRefresh.value = await invoke("get_levels", { fsHomepath: fsHomepath.value })
      levels.value = levelsLastRefresh.value
      await invoke("extract_levelshots_to_cache", { levels: levelsLastRefresh.value })
      levelshots.value = await invoke("get_cached_levelshots")

      if (levelshots.value) {
        for (var key in levelshots.value) {
          levelshots.value[key] = convertFileSrc(levelshots.value[key])
        }
      }
    } catch (err) {
      emit("errorAlert", ensureError(err).message)
    }

    loading.value = false
    loadingEvent.value = ""
    const executionTime = performance.now() - startTime

    info(`${totalMaps.value} levels read from ${fsHomepath.value} in ${parseFloat((executionTime / 1000).toFixed(2))} seconds`)
  }

  const searchQuery = ref("")

  watch(searchQuery, async (newSearch, _oldSearch) => {
    selectedLevel.value = null
    lastSelectedLevel.value = null
    sortDesc.value = false
    currentSort.value = ""

    let query = newSearch.toLowerCase()

    let filteredMaps: Level[] = []

    for (let i = 0; i < levelsLastRefresh.value.length; i++) {
      if (levelsLastRefresh.value[i].level_name.toLowerCase().includes(query)) {
        filteredMaps.push(levelsLastRefresh.value[i])
      }
    }
    levels.value = filteredMaps
  })

  const sortDesc = ref(false)
  const currentSort = ref("")

  const selectedLevel = ref<Nullable<Level>>(null)
  const lastSelectedLevel = ref<Nullable<Level>>(null)

  const launchArg = ref<string[]>([])
  const gameSelect = ref('sp')
  const teamSelect = ref<'Free' | 'Red' | 'Blue'>('Free')
  const difficulty = ref<1 | 2 | 3 | 4 | 5>(5)
  const cheats = ref(false)

  function getQ3ClientType() {
    let activeClient = props.config.q3_clients?.filter((c) => c.active)[0]

    if (activeClient) {
      if (['odfe.x64.exe', 'odfe.x64'].includes(activeClient.name.toLowerCase())) {
        return 'df'
      } 
      if (['cnq3-x64', 'cnq3-x64.exe', 'cnq3-x86.exe'].includes(activeClient.name.toLowerCase())) {
        return 'cpma'
      } 
      return 'q3'
    } else {
      return 'no active client'
    }
  }
  
  const selectedMapIndex = computed(() => {
    if (selectedLevel.value) {
      return levels.value.indexOf(selectedLevel.value)
    }
    return -1
  })

  function getLevelIndex(d: Level) { return levels.value.indexOf(d) }

  watch(selectedLevel, (_newSelectedMap, _old) => {

    //emitConnect()
  })

  function emitConnect() {
    if (selectedLevel.value != null) {
      emit("emitConnectArgs", launchArg.value)
    }
  }

  function escapeButton() {
    lastSelectedLevel.value = null
    selectedLevel.value = null
  }

  function spawnQuake() {
    if (selectedLevel.value && gameSelect.value) {

      let args = new Map<string, string>([
        ['sp', '+spmap'],
        ['spc', '+spdevmap'],
        ['1v1', '+map'],
        ['1v1c', '+devmap'],
        ['ffa', '+spmap'],
        ['ffac', '+spdevmap'],
        ['tdm', '+map'],
        ['tdmc', '+devmap'],
        ['ctf', '+map'],
        ['ctfc', '+devmap'],
        ['cpm', '+cpm'],
        ['cpmc', '+devcpm'],
        ['vq3', '+vq3'],
        ['vq3c', '+devvq3']
      ])


      let gametype = '0'
      if (gameSelect.value == 'ffa') { gametype = '0' }
      if (gameSelect.value == '1v1') { gametype = '1' }
      if (gameSelect.value == 'sp') { gametype = '2' }
      if (gameSelect.value == 'tdm') { gametype = '3' }
      if (gameSelect.value == 'ctf') { gametype = '4' }

      let launch = args.get(gameSelect.value + (cheats.value ? 'c' : '' ))!

      let arg = [launch, selectedLevel.value.level_name, '+set', 'g_gametype', gametype]

      arg.push(...['+set', gametype == '2' ? 'g_spskill' : 'skill', difficulty.value.toString()])
    
      if (gameSelect.value != 'tdm' && gameSelect.value != 'ctf' && getQ3ClientType() != 'df') {
        bots_team_free.value.forEach(bot => {
          arg.push(...['+addbot', bot.name, bot.difficulty.toString(), bot.team])
        });
      }

      if (gameSelect.value == 'tdm' || gameSelect.value == 'ctf' && getQ3ClientType() != 'df') {
        bots_team_red.value.forEach(bot => {
          arg.push(...['+addbot', bot.name, bot.difficulty.toString(), bot.team])
        });

        bots_team_blue.value.forEach(bot => {
          arg.push(...['+addbot', bot.name, bot.difficulty.toString(), bot.team])
        });

      }
      
      arg.push(...['+wait', '5'])
      arg.push(...['+team', teamSelect.value])
      
      emit("emitConnectArgs", arg)
      emit("spawnQuake", componentName.value)
    }
  }

  const getVirtualRows = computed(() => {
    return levels.value.slice(virtualStartIndex.value, virtualEndIndex.value)
  })

  const totalMaps = computed(() => {
    return levels.value.length
  })

  function keySelect(increment: number) {
    if (selectedLevel.value == null || (selectedMapIndex.value + increment) < 0 || (selectedMapIndex.value + increment) == levelsLength.value) {
      return
    }

    selectedLevel.value = levels.value[selectedMapIndex.value + increment]

    nextTick(() => {
      // the real dom is now updated (document.)
      let selected = document.getElementById("selected")

      if (selected) {
        selected.scrollIntoView({ behavior: "instant" as ScrollBehavior, block: "start", inline: "nearest" })
        selected.focus()
      }
    })
  }

  const {
    handleClick,
    dblClickHappenedOnSameObject,
    resetDblClickTimeout
  } = useClickRow(selectedLevel, lastSelectedLevel);

  function clickLevel(clicked: Level) {
    handleClick(clicked)

    if (dblClickHappenedOnSameObject.value) {
      spawnQuake()
      resetDblClickTimeout()
    }
  }

  const levelshots = ref<Nullable<{ [key: string]: string }>>(null)

  const levelsLength = computed(() => { return levels.value.length }) 

  const { setScroller,
    translateY,
    virtualHeight,
    marginTop,
    virtualStartIndex,
    virtualEndIndex 
  } = useVirtualScroll('levelTable', levelsLength);

  setScroller({rowHeight: 96, overscan: 8})

  onMounted(async () => {
    emit("emitComponentName", componentName.value)
    await getLevels()
  })

  onActivated(async () => {
    emit("emitComponentName", componentName.value)
    emitConnect()
  })

</script>


<template>
  <div class="table-header-base" style="height: 40px;">
    <div class="table-header-right">
      <button class="refresh-button" :class="{'pak0-only': showPak0LevelsOnly}" @click="pak0LevelsOnly();">Q3A</button>
      <input class="search" type="text" placeholder="search" v-model="searchQuery" />
    </div>
    <div class="table-header-left">
      <button class="connect-button" :disabled="!selectedLevel || !gameSelect" @click="spawnQuake()">Connect</button>
      <button class="refresh-button" @click="getLevels()">Refresh</button>
    </div>

    <!-- <div class="table-column-header">
      <span style="width: 12%; text-align: left"></span>
      <span style="width: 44%; text-align: left">map<span :class="getArrowSort('level_name')" @click="sortMaps('level_name')" /></span>
    </div> -->
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
    <div v-if="!loading && fsHomepath" :style="{ height: virtualHeight + 'px' }">
      <div class="main" :style="{ transform: 'translateY(' + translateY + 'px)', marginTop: marginTop + 'px' }">
        <div
          v-for="level in getVirtualRows"
          class="level-row"
          :style="getLevelIndex(level) % 2 ? 'background-color: rgba(23, 32, 45, 0.3);' : ''"
          :key="level.path + level.pk3_name + level.level_name"
          :id="level === selectedLevel ? 'selected' : `level-${getLevelIndex(level)}`"
          tabindex="0"
          @click="clickLevel(level)"
        >
          <div class="map-row">
            <img height="96px" width="128px" :src="levelshots![level.level_name.toLowerCase()]" @error="setDefaultImage"/>
            <h3 style="width: 44%; text-align: left; white-space: nowrap; overflow: hidden; margin-left: 24px;">
              {{ level.level_name }}
            </h3>
          </div>
        </div>
      </div>
    </div>
    <div v-if="loading">
      <Loading :position="'center'" :message="loadingEvent" :size="90" />
      <div v-for="(_, index) in 48" class="row" :style="index % 2 ? 'background-color: rgba(23, 32, 45, 0.3);' : ''"></div>
    </div>
    <div v-if="!fsHomepath">
      <div class="center"><button class="select-path-button" @click="pickFsHomepath()">Set fs_homepath</button></div>
      <div v-for="(_, index) in 48" class="row" :style="index % 2 ? 'background-color: rgba(23, 32, 45, 0.3);' : ''"></div>
    </div>
  </div>

  <div class="table-footer">
    <div class="table-footer-right">
      <span class="footer-data-right" v-if="searchQuery.length == 0">maps: {{ levelsLastRefresh.length }}</span>
      <span class="footer-data-right" v-if="searchQuery.length > 0">maps: {{ levels.length }}</span>
    </div>
    <div class="table-footer-left">
      <img src="../assets/icons/q3-white.svg" class="footer-icon" @click="pickFsHomepath()" />
      <span class="footer-url">{{ fsHomepath }}</span>
    </div>
  </div>

  <div v-if="selectedLevel" class="game-setup no-select">
    <h2 style="text-align: center;">Game Setup</h2>
    <div style="text-align: center; margin-bottom: 10px;">
      <button class="setup-button" v-if="getQ3ClientType() != 'df'" :style="gameSelect == 'sp' ? 'background-color: rgba(0, 143, 168, 0.514);': ''" @click="gameSelect = 'sp'">SP</button>
      <button class="setup-button" v-if="getQ3ClientType() != 'df'" :style="gameSelect == '1v1' ? 'background-color: rgba(0, 143, 168, 0.514);': ''" @click="gameSelect = '1v1'">1v1</button>
      <button class="setup-button" v-if="getQ3ClientType() != 'df'" :style="gameSelect == 'ffa' ? 'background-color: rgba(0, 143, 168, 0.514);': ''" @click="gameSelect = 'ffa'">FFA</button>
      <button class="setup-button" v-if="getQ3ClientType() != 'df'" :style="gameSelect == 'tdm' ? 'background-color: rgba(0, 143, 168, 0.514);': ''" @click="gameSelect = 'tdm'; teamSelect = teamSelect == 'Free' ? 'Red': teamSelect">TDM</button>
      <button class="setup-button" v-if="getQ3ClientType() != 'df'" :style="gameSelect == 'ctf' ? 'background-color: rgba(0, 143, 168, 0.514);': ''" @click="gameSelect = 'ctf'; teamSelect = teamSelect == 'Free' ? 'Red': teamSelect">CTF</button>
      <!-- <button class="setup-button" v-if="getQ3ClientType() != 'df'" :style="gameSelect == '1fctf' ? 'background-color: rgba(0, 143, 168, 0.514);': ''" @click="gameSelect = '1fctf'">1FCTF</button> -->

      <button class="setup-button" v-if="getQ3ClientType() == 'df'" :style="gameSelect == 'vq3' ? 'background-color: rgba(0, 143, 168, 0.514);': ''" @click="gameSelect = 'vq3'">VQ3</button>
      <button class="setup-button" v-if="getQ3ClientType() == 'df'" :style="gameSelect == 'cpm' ? 'background-color: rgba(0, 143, 168, 0.514);': ''" @click="gameSelect = 'cpm'">CPM</button>

    </div>
    <div style="text-align: center; margin-bottom: 10px;">
      <button class="setup-button" v-if="getQ3ClientType() != 'df'" style="font-size: 100%;" :style="difficulty == 1 ? 'background-color: rgba(0, 143, 168, 0.514);': ''" @click="difficulty = 1">i can win</button>
      <button class="setup-button" v-if="getQ3ClientType() != 'df'" style="font-size: 100%;" :style="difficulty == 2 ? 'background-color: rgba(0, 143, 168, 0.514);': ''" @click="difficulty = 2">bring it on</button>
      <button class="setup-button" v-if="getQ3ClientType() != 'df'" style="font-size: 100%;" :style="difficulty == 3 ? 'background-color: rgba(0, 143, 168, 0.514);': ''" @click="difficulty = 3">hurt me plenty</button>
      <button class="setup-button" v-if="getQ3ClientType() != 'df'" style="font-size: 100%;" :style="difficulty == 4 ? 'background-color: rgba(0, 143, 168, 0.514);': ''" @click="difficulty = 4">hardcore</button>
      <button class="setup-button" v-if="getQ3ClientType() != 'df'" style="font-size: 100%;" :style="difficulty == 5 ? 'background-color: rgba(0, 143, 168, 0.514);': ''" @click="difficulty = 5">nightmare!</button>

    </div>

    <div style="display: flex; height: 40%; user-select: none;">
    
      <img :src="levelshots![selectedLevel.level_name.toLowerCase()]" @error="setDefaultImage" style="width: 50%;">
      <div style="width: 50%; text-align: center; overflow: hidden scroll; padding: 4px 0px 4px 4px;" v-if="gameSelect != 'ctf' && gameSelect != 'tdm'">
        <div v-for="b in bots_team_free" style="white-space: nowrap;"> 
          <button @click="bots_team_free.splice(bots_team_free.indexOf(b), 1)" class="close-button"><img src="../assets/icons/x.svg" width="8px" /></button>
          <label class="bot-button" @click="b.name = Q3_BOT_NAMES[(Q3_BOT_NAMES.indexOf(b.name)+1) % 23]">{{ b.name }}</label>
          <label class="bot-button" style="margin-left: 6px;" @click="b.difficulty = (b.difficulty % 5) + 1">{{ b.difficulty }}</label>
        </div>
      </div>
      <div style="width: 25%; text-align: left; overflow: hidden scroll; background-color: rgba(255, 0, 0, 0.2); padding: 4px 0px 4px 4px;"  v-if="gameSelect == 'ctf' || gameSelect == 'tdm'">
        <div v-for="b in bots_team_red" style="white-space: nowrap;"> 
          <button @click="bots_team_red.splice(bots_team_red.indexOf(b), 1)" class="close-button"><img src="../assets/icons/x.svg" width="8px" /></button>
          <label class="bot-button" @click="b.name = Q3_BOT_NAMES[(Q3_BOT_NAMES.indexOf(b.name)+1) % 23]">{{ b.name }}</label>
          <label class="bot-button" style="margin-left: 6px;" @click="b.difficulty = (b.difficulty % 5) + 1">{{ b.difficulty }}</label>
        </div>
      </div>
      <div style="width: 25%; text-align: left; overflow: hidden scroll; background-color: rgba(0, 0, 255, 0.2); padding: 4px 0px 4px 4px;" v-if="gameSelect == 'ctf' || gameSelect == 'tdm'">
        <div v-for="b in bots_team_blue" style="white-space: nowrap;"> 
          <button @click="bots_team_blue.splice(bots_team_blue.indexOf(b), 1)" class="close-button"><img src="../assets/icons/x.svg" width="8px" /></button>
          <label class="bot-button" @click="b.name = Q3_BOT_NAMES[(Q3_BOT_NAMES.indexOf(b.name)+1) % 23]">{{ b.name }}</label>
          <label class="bot-button" style="margin-left: 6px;" @click="b.difficulty = (b.difficulty % 5) + 1">{{ b.difficulty }}</label>
        </div>
      </div>
    </div>

    <div style="display: flex; margin: 10px 0;">
      <div style="width: 50%; text-align: center;">
        <button class="setup-button" style="font-size: 100%;" :style="cheats ? 'background-color: rgba(0, 143, 168, 0.514);': ''" @click="cheats = !cheats">cheats</button>
        <button v-if="gameSelect == 'ctf' || gameSelect == 'tdm'" class="setup-button" style="font-size: 100%;" :style="teamSelect == 'Red' ? 'background-color: rgba(255, 0, 0, 0.2)': 'background-color: rgba(0, 0, 255, 0.2);'" @click="teamSelect = teamSelect == 'Red' ? 'Blue' : 'Red'">My Team: {{ teamSelect }}</button>
      </div>
      <div style="width: 50%; text-align: center;" v-if="gameSelect != 'ctf' && gameSelect != 'tdm'">
        <button v-if="gameSelect != 'ctf' && gameSelect != 'tdm'" class="setup-button" style="font-size: 100%; display: inline-block;" @click="bots_team_free.push({name: 'sarge', difficulty: difficulty, team: 'Free'})">+bot</button>
      </div>
      <div style="width: 25%; text-align: center;"  v-if="gameSelect == 'ctf' || gameSelect == 'tdm'">
        <button v-if="gameSelect == 'ctf' || gameSelect == 'tdm'" class="setup-button" style="font-size: 100%; display: inline-block;" @click="bots_team_red.push({name: 'sarge', difficulty: difficulty, team: 'Red'})">+bot</button>
      </div>
      <div style="width: 25%; text-align: center;" v-if="gameSelect == 'ctf' || gameSelect == 'tdm'">
        <button v-if="gameSelect == 'ctf' || gameSelect == 'tdm'" class="setup-button" style="font-size: 100%; display: inline-block;" @click="bots_team_blue.push({name: 'sarge', difficulty: difficulty, team: 'Blue'})">+bot</button>
      </div>
    </div>


    <div style="text-align: center;">
      <button class="setup-button" :disabled="!selectedLevel || !gameSelect"  @click="spawnQuake()">Connect</button>
    </div>

  </div>

</template>

<style scoped>

  .pak0-only-button {
    cursor: pointer;
    padding: 4px 15px;
    margin: 0px 4px;
    background: url('../assets/icons/q3-white.svg') center center no-repeat;
    background-size: 78%;
  }

  .pak0-only-button:hover {    
    background-color: var(--main-bg);
    border-radius: 0.2rem;
  }

  .pak0-only {    
    background-color: rgba(0, 143, 168, 0.514);
    border-radius: 0.2rem;
    cursor: pointer;
  }

  .single-player {
    height: calc(100% - 72px)
  }

  .close-button {
    background: rgba(0, 0, 0, 0);
    border: 1px solid var(--main-bg);;
    border-radius: 0.2rem;
    margin: 0px 8px 8px 0px;
  }

  .close-button:hover {
    background-color: var(--main-bg);;
    cursor: pointer;
  }

  .bot-button {
    background: rgba(0, 0, 0, 0);
    border: 1px solid var(--main-bg);;
    border-radius: 0.2rem;
    padding: 3px 5px;
    font-size: 80%;
  }

  .bot-button:hover {
    background-color: var(--main-bg);;
    cursor: pointer;
  }

  .setup-button {
    background-color: rgba(0, 0, 0, 0);
    color: white;
    border: 1px solid var(--main-bg); /* Green */
    border-radius: 0.2rem;
    cursor: pointer;
    font-size: 150%;
    padding: 2px 10px 2px 10px;
    font-weight: 400;
    margin-right: 4px;
    /* letter-spacing: 1px; */
  }

  .setup-button:disabled,
  .setup-button[disabled]{
    background-color: rgba(0, 0, 0, 0);
    color: var(--main-bg);;
    border: 1px solid var(--main-bg);; /* Green */
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


  .game-setup {
    position: absolute;
    top: 96px;
    right: 52px;
    bottom: 82px;
    width: 50%;
    /* border-radius: 0.2rem; */
    background-color: rgba(0, 0, 0, 0.15);
    border: 1px solid #00ffff;
    backdrop-filter: blur(10px);
    color: #fff;
    /* height: max-content; */
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

  .level-row {
    min-height: 96;
    color: white;
    overflow: hidden;
    outline: none;
  }


</style>
