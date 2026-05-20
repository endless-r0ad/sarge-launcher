<script setup lang="ts">
  import Modal from '@/components/Modal.vue'
  import Loading from '@/components/Loading.vue'
  import ClientProfile from '@/components/ClientProfile.vue'
  import { ref, onMounted, onActivated, onDeactivated } from 'vue'
  import { ensureError } from '@/utils/util'
  import { useConfig } from '@/composables/config'
  import { useClient } from '@/composables/client'
  import { removeDotSuffix, newCustomServer } from '@/utils/util'
  import { useAppData } from '@/composables/appdata'
  import { invoke } from '@tauri-apps/api/core'
  import { type Quake3Server } from '@/models/server'

  const emit = defineEmits<{spawnQuake: [string[]], emitComponentName: [string], alert: [string, string]}>()
  const props = defineProps<{ latestGithubVersion: string | null }>()

  const componentName = ref('Client Setup')
  const appVersion = 'v0.3.1'
  const updateAvailable = ref(props.latestGithubVersion && appVersion != props.latestGithubVersion)
  const { config } = useConfig()
  const { appdata } = useAppData()
  const { pickClient, activeClient, deleteQ3Client, toggleQ3Client, clientIsOverridden } = useClient()

  const hoveredCard = ref('')

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

  const favoritedServers = ref<Quake3Server[]>([])
  const loadingFaveServers = ref(false)

  async function refreshFavoriteServers() {
    loadingFaveServers.value = true
    favoritedServers.value = []

    let servers = appdata.value.pinned;

    servers.forEach(address => {
      let ip_and_port = address.split(':')
      let faveServer: Quake3Server = newCustomServer(ip_and_port, address)
      favoritedServers.value.push(faveServer)
    });

    if (favoritedServers.value.length === 0) { 
      loadingFaveServers.value = false
      return 
    } 

    try {

      favoritedServers.value = await invoke('refresh_all_servers', 
                { 
                  allServers: favoritedServers.value, 
                  numThreads: (config.value.server_browser_threads == 0 ? 1 : config.value.server_browser_threads),
                  timeout: config.value.server_timeout
                })
    }
    catch(err) {
      emit('alert', 'error', ensureError(err).message)
    }

    favoritedServers.value.sort((a, b) => {
      if (a['playersconnected'] > b['playersconnected']) {
        return -1;
      }
      if (a['playersconnected'] < b['playersconnected']) {
        return 1;
      }
      return 0;
    });

    loadingFaveServers.value = false

  }

  const showClientProfile = ref(false)
  const showQ3Config = ref(false)

  function showIfActiveClient() {
    if (activeClient.value) {
      return true
    }
    emit('alert', 'info', 'Link a Quake 3 client first')
    return false
  }

  onMounted(() => emit('emitComponentName', componentName.value))

  onActivated(async() => {
    emit('emitComponentName', componentName.value)
    await refreshFavoriteServers() 
  })

  onDeactivated(() => hoveredCard.value = '')
</script>

<template>
  <Teleport to="#modal">
    <Modal v-if="showClientProfile" :popup-type="'center'" @close="showClientProfile=false">
      <ClientProfile :profiledClient="activeClient!" @deleteClient="showClientProfile=false; deleteQ3Client(activeClient!);"/>
    </Modal>
    <Modal v-if="config.welcome_message || updateAvailable" :popupType="'center'" @close="config.welcome_message = false; updateAvailable = false">
      <div style="width: 400px">
        <img style="position: absolute; left: 15%; top: 4%" src="../assets/icons/sarge.svg" />
        <h2 style="position: absolute; right: 15%; top: 4%">SARGE LAUNCHER</h2>
        <a v-if="props.latestGithubVersion && appVersion != props.latestGithubVersion" 
          class="link" 
          href="https://github.com/endless-r0ad/sarge-launcher/releases" 
          target="_blank">
          <p style="position: absolute; right: 42%; top: 16%; font-size: 75%; color: #00ffff;">
            update available
          </p>
        </a>
        <a class="link" href="https://github.com/endless-r0ad/sarge-launcher" target="_blank">
          <p :style="`position: absolute; right: ${props.latestGithubVersion && appVersion != props.latestGithubVersion ? 29 : 15}%; top: 16%; font-size: 75%;`">
            {{ appVersion }}
          </p>
        </a>
        <a v-if="props.latestGithubVersion && appVersion != props.latestGithubVersion" 
          class="link" 
          href="https://github.com/endless-r0ad/sarge-launcher/releases" 
          target="_blank">
          <p style="position: absolute; right: 15%; top: 16%; font-size: 75%; color: #00ffff;">
            -> {{ props.latestGithubVersion }}
          </p>
        </a>
        <p style="margin-top: 72px">
          Sarge Launcher is a utility for Quake 3 Arena and Q3A mods that provides some useful features for both n00bs and
          veterans. If you are new, you should purchase the game first, then update your client to ioquake3 or quake3e.
        </p>
        <p>
          Sarge Launcher is not responsible for installing any mods or downloading any content for Q3A -
          YOU are solely responsible for that!
        </p>
      </div>
    </Modal>
  </Teleport>

  <div class="client-grid no-select" draggable="false">
    <div
      class="grid-bg plus-bg grow"
      @mouseover="hoveredCard = 'link client'"
      @mouseleave="hoveredCard = ''"
      style="cursor: pointer; background-color: var(--secondary-bg); grid-column: 1; grid-row: 1"
    >
      <div v-if="hoveredCard == 'link client'" class="tint" @click="pickQ3Client()">
        <span class="center card-name">{{ hoveredCard }}</span>
      </div>
    </div>
    
    <div class="grid-bg" style="grid-column: 1 / 4; grid-row: 2 / 4" >
      <div class="client-container">
        <div
          v-for="(client, index) in config.q3_clients"
          class="client"
          :class="client.gamename"
          :key="client.exe_path"
          :style="index % 2 ? 'background-color: rgba(23, 32, 45, 0.3);' : ''"
          :id="client.exe_path === activeClient?.exe_path ? 'selected' : 'c'"
          @click="toggleQ3Client(client)"
        >
          {{ removeDotSuffix(client.name) }}
          <h6 v-if="clientIsOverridden(client)" style="font-size: 70%; 
    color: orange; 
    font-weight: 800;
    margin: -24px 0 0px 0;">
            {{ client.gamename }}
      </h6>
        </div>
      </div>
    </div>
    <div class="grid-bg" style="grid-column: 4 / 6; grid-row: 1 / 3" >
      <div v-if="loadingFaveServers" style="position: relative; height: 100%;">
        <Loading :position="'center'" :size="90" />
      </div>
      <div v-else class="client-container">
        <div
          v-for="(server, index) in favoritedServers"
          class="server"
          :key="server.address"
          :style="index % 2 ? 'background-color: rgba(23, 32, 45, 0.3);' : ''"
        >
          <span style="width: 11%; padding-right: 12px;">{{ server.game }}</span>
          <span v-html="server.hostcolored" style="width: 36%; padding-right: 12px;"></span>
          <span style="width: 10%" class="data">{{ server.playersconnected }}/{{ server.maxclients }}</span>

        </div>
      </div>
    </div>
    <div
      class="grid-bg q3-bg grow"
      @mouseover="hoveredCard = 'config'"
      @mouseleave="hoveredCard = ''"
      style="grid-column: 1 / 4; grid-row: 4"
    >
      <div v-if="hoveredCard == 'config'" class="tint" @click="showQ3Config=showIfActiveClient()">
        <span class="center card-name" draggable="false">{{ activeClient?.name }} q3config.cfg</span>
      </div>
    </div>
    <div
      class="grid-bg play-bg grow"
      @mouseover="hoveredCard = 'launch client'"
      @mouseleave="hoveredCard = ''"
      style="cursor: pointer; background-color: var(--secondary-bg); grid-column: 2; grid-row: 1"
    >
      <div v-if="hoveredCard == 'launch client'" class="tint" @click="emit('spawnQuake', [])">
        <span class="center card-name">{{ hoveredCard }}</span>
      </div>
    </div>
    <div
      class="grid-bg settings-bg grow"
      @mouseover="hoveredCard = 'client profile'"
      @mouseleave="hoveredCard = ''"
      style="cursor: pointer; background-color: var(--secondary-bg); grid-column: 3; grid-row: 1"
    >
      <div v-if="hoveredCard == 'client profile'" class="tint" @click="showClientProfile=showIfActiveClient()">
        <span class="center card-name">{{ hoveredCard }}</span>
      </div>
    </div>

    <div
      class="grid-bg sp-bg grow"
      @mouseover="hoveredCard = 'Single Player'"
      @mouseleave="hoveredCard = ''"
      style="grid-column: 4; grid-row: 3"
    >
      <router-link to="/singleplayer" class="link">
        <div v-if="hoveredCard == 'Single Player'" class="tint">
          <span class="center card-name">{{ hoveredCard }}</span>
        </div>
      </router-link>
    </div>
    <div
      class="grid-bg servers-bg grow"
      @mouseover="hoveredCard = 'Servers'"
      @mouseleave="hoveredCard = ''"
      style="grid-column: 4; grid-row: 4"
    >
      <router-link to="/server" class="link">
        <div v-if="hoveredCard == 'Servers'" class="tint">
          <span class="center card-name">{{ hoveredCard }}</span>
        </div>
      </router-link>
    </div>
    <div
      class="grid-bg demos-bg grow"
      @mouseover="hoveredCard = 'Demos'"
      @mouseleave="hoveredCard = ''"
      style="grid-column: 5; grid-row: 3"
    >
      <router-link to="/demo" class="link">
        <div v-if="hoveredCard == 'Demos'" class="tint">
          <span class="center card-name">{{ hoveredCard }}</span>
        </div>
      </router-link>
    </div>
    <div
      class="grid-bg resources-bg grow"
      @mouseover="hoveredCard = 'Resources'"
      @mouseleave="hoveredCard = ''"
      style="grid-column: 5; grid-row: 4"
    >
      <router-link to="/resource" class="link">
        <div v-if="hoveredCard == 'Resources'" class="tint">
          <span class="center card-name">{{ hoveredCard }}</span>
        </div>
      </router-link>
    </div>
  </div>
</template>

<style scoped>
  .client-grid {
    height: 100%;
    display: grid;
    grid-template-columns: repeat(5, minmax(111px, 20%));
    grid-template-rows: repeat(4, minmax(74px, 25%));
    grid-gap: 16px;
    padding: 4px;
    color: white;
    user-select: none;
  }

  .client-container {
    height: 100%;
    width: 100%;
    overflow-y: scroll;
    overflow-x: hidden;
    font-weight: bold;
  }

  .client {
    width: 100%; 
    height: 48px; 
    padding: 12px 12px 12px 48px; 
    line-height: 48px;
    font-size: 120%;
  }

  .server {
    width: 100%; 
    height: 32px; 
    padding: 12px 12px 12px 32px; 
    line-height: 32px;
  }

  .active {
    background-color: #fff;
  }

  .grid-bg {
    background-color: var(--secondary-bg);
    background-repeat: no-repeat;
    background-size: 100%;
    background-position: center center;
    color: white;
  }

  .q3-bg {
    background-image: url('../assets/icons/q3-white.svg');
    background-size: 10%;
  }

  .plus-bg {
    background-image: url('../assets/icons/plus.svg');
    background-size: 20%;
  }

  .play-bg {
    background-image: url('../assets/icons/play.svg');
    background-size: 20%;
  }

  .settings-bg {
    background-image: url('../assets/icons/settings.svg');
    background-size: 24%;
  }

  .sp-bg {
    background-image: url('../assets/icons/single-player.svg');
    background-size: 30%;
  }

  .servers-bg {
    background-image: url('../assets/icons/globe.svg');
    background-size: 30%;
  }

  .demos-bg {
    background-image: url('../assets/icons/replay.svg');
    background-size: 30%;
  }

  .resources-bg {
    background-image: url('../assets/icons/new-window.svg');
    background-size: 30%;
  }

  .grow {
    transition: all 0.35s ease-in-out;
  }

  .grow:hover {
    transform: scale(1.02);
    cursor: pointer;
  }

  .tint {
    min-height: 100%;
    min-width: 100%;
    background-color: rgba(0, 0, 0, 0.6);
  }

  .center {
    position: absolute;
    left: 50%;
    top: 50%;
    transform: translate(-50%, -50%);
    text-align: center;
    min-width: 100%;
  }

  .card-name {
    font-size: 150%;
    font-weight: bold;
  }

  .link {
    color: white;
    z-index: 998;
  }


</style>
