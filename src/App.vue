<script setup lang="ts">
  import Loading from '@/components/Loading.vue'
  import Header from '@/components/Header.vue'
  import Popup from '@/components/Popup.vue'
  import Sidebar from '@/components/Sidebar.vue'
  import { type Nullable, ensureError, tempConfig, tempAppData } from '@/utils/util'
  import { ref, onMounted } from 'vue'
  import { invoke } from '@tauri-apps/api/core'
  import { info, error } from '@tauri-apps/plugin-log'
  import { type Config, type AppData } from '@/models/config'
  import { type Q3Executable } from '@/models/client'

  const config = ref<Config>(tempConfig())
  const appData = ref<AppData>(tempAppData())

  onMounted(async () => {
    try {
      config.value = await invoke('get_config')
      appData.value = await invoke('get_appdata')
      appData.value.pinned = new Set(appData.value.pinned)
      appData.value.custom = new Set(appData.value.custom)
      appData.value.trash = new Set(appData.value.trash)
      appData.value.trash_ip = new Set(appData.value.trash_ip)

      if (config.value.q3_clients.length > 0) {
        let client: Q3Executable = config.value.q3_clients[0]
        client.active = true
        activeClient.value = client
      }

      await new Promise((r) => setTimeout(r, 500))
      isMounted.value = true
    } catch (err) {
      showErrorAlert(ensureError(err).message)
    }
  })

  async function saveConfig() {
    try {
      showSettings.value = false
      await invoke('save_config', { updatedConfig: config.value })
    } catch (err) {
      showErrorAlert(ensureError(err).message)
    }
  }

  async function mutateConfig(newConfig: Config) {
    config.value = newConfig
    await saveConfig()
  }

  async function mutateAppData(newAppData: AppData) {
    appData.value = newAppData

    const appDataForBackend = {
      ...newAppData,
      pinned: Array.from(newAppData.pinned),
      custom: Array.from(newAppData.custom),
      trash: Array.from(newAppData.trash),
      trash_ip: Array.from(newAppData.trash_ip),
    }
    await invoke('save_app_data', { updatedData: appDataForBackend })
  }

  const isMounted = ref(false)
  const q3ClientProcessId = ref<Nullable<number>>(null)
  const currentComponent = ref('')

  function getComponentName(componentName: string) { currentComponent.value = componentName }

  const showSettings = ref(false)

  function toggleSettings() { showSettings.value = !showSettings.value }

  const showAlertPopup = ref(false)
  const popupType = ref('')
  const alertMessage = ref('')

  function showInfoAlert(infoMsg: string) {
    info(infoMsg)
    showAlertPopup.value = true 
    popupType.value = 'info'
    alertMessage.value = infoMsg
  }

  function showErrorAlert(err: string) {
    error(err)
    showAlertPopup.value = true
    popupType.value = 'error'
    alertMessage.value = err
  }

  function closeAlert() { 
    showAlertPopup.value = false
    popupType.value = ''
    alertMessage.value = '' 
  }

  const activeClient = ref<Nullable<Q3Executable>>(null)

  async function addQ3Client(client: Q3Executable) {
    if (config.value.q3_clients.length > 0) {
      if (config.value.q3_clients.some((c) => c.exe_path === client.exe_path)) {
        showInfoAlert('client already added')
        return
      }
      config.value.q3_clients.map((client) => (client.active = false))
    }
    client.active = true
    config.value.q3_clients.push(client)
    activeClient.value = client
    await saveConfig()
  }

  async function toggleQ3Client(client: Q3Executable) {
    if (client == activeClient.value) {
      return
    }
    config.value.q3_clients.map((c) => {
      c.active = c.exe_path == client.exe_path
    })
    activeClient.value = client
    await saveConfig()
  }

  async function deleteQ3Client(client: Q3Executable) {
    if (config.value.q3_clients.length == 0) {
      return
    }

    config.value.q3_clients = config.value.q3_clients.filter((c) => c.exe_path != client.exe_path)
    if (config.value.q3_clients.length == 0) {
      activeClient.value = null
    }
    if (config.value.q3_clients.length > 0 && !config.value.q3_clients.some((x) => x.active)) {
      config.value.q3_clients[0].active = true
      activeClient.value = config.value.q3_clients[0]
    }
    await saveConfig()
  }

  async function spawnQuake(launchArgs: string[]) {
    if (config.value.manage_q3_instance && q3ClientProcessId.value !== null) {
      try {
        await invoke('kill_q3_client', { processId: q3ClientProcessId.value })
      } catch (err) {
        showErrorAlert(ensureError(err).message)
      }
    }

    try {
      q3ClientProcessId.value = await invoke('spawn_quake', {
        activeClient: activeClient.value,
        q3Args: ['+set', 'fs_basepath', activeClient.value?.parent_path].concat(launchArgs),
        manageInstance: config.value.manage_q3_instance,
      })
    } catch (err) {
      const e: Error = ensureError(err)
      if (e.message.includes('expected struct Q3Executable') || e.message.includes('missing required key activeClient')) {
        showInfoAlert('Please configure a Quake 3 client to launch')
      } else {
        showErrorAlert(e.message)
      }
    }
  }

  async function exitApp() {
    await invoke('exit_app')
  }
</script>

<template>
  <Loading v-if="!isMounted" :position="'center'" :message="'loading...'" :size="90" />

  <Header
    v-if="isMounted"
    :currentView="currentComponent"
    :config="config"
    :activeClient="activeClient"
    @spawnQuake="spawnQuake"
    @addQ3Client="addQ3Client"
    @toggleQ3Client="toggleQ3Client"
    @deleteQ3Client="deleteQ3Client"
    @errorAlert="showErrorAlert"
  />

  <Sidebar v-if="isMounted" :showSettings="showSettings" @toggleSettings="toggleSettings" @exitApp="exitApp" />

  <div v-if="isMounted" class="main-view">
    <router-view v-slot="{ Component }">
      <KeepAlive exclude="Home">
        <component
          :is="Component"
          :config="config"
          :appData="appData"
          :showUnreachableServers="config.show_unreachable"
          :showTrashedServers="config.show_trashed_servers"
          @mutateConfig="mutateConfig"
          @mutateAppData="mutateAppData"
          @spawnQuake="spawnQuake"
          @addQ3Client="addQ3Client"
          @emitComponentName="getComponentName"
          @errorAlert="showErrorAlert"
          @infoAlert="showInfoAlert"
        />
      </KeepAlive>
    </router-view>
  </div>

  <div id="popup">
    <Popup v-if="showAlertPopup" :popupType="popupType" @cancelModal="closeAlert" @executeModal="closeAlert">{{ alertMessage }}</Popup>
    <Popup v-if="showSettings" :popupType="'center'" @executeModal="saveConfig" @cancelModal="saveConfig">
      <div class="item"><input type="checkbox" v-model="config.manage_q3_instance" /><label class="ml-1">Manage Q3 Instance</label></div>
      <div class="item">
        <input type="checkbox" v-model="config.show_unreachable" /><label class="ml-1">Show Unreachable Servers</label>
      </div>
      <div class="item">
        <input type="checkbox" v-model="config.show_trashed_servers" /><label class="ml-1">Show Trashed Servers</label>
      </div>

      <div class="config-plus">
        +<label class="ml-1">Browser Threads - {{ config.server_browser_threads == 0 ? 1 : config.server_browser_threads }}</label>
      </div>
      <div class="item">
        <input type="range" min="0" max="120" step="5" value="60" class="slider" v-model.number="config.server_browser_threads" />
      </div>

      <div class="config-plus">
        +<label class="ml-1">Server Timeout - {{ config.server_timeout }}ms</label>
      </div>
      <div class="item">
        <input type="range" min="200" max="1000" step="100" value="400" class="slider" v-model.number="config.server_timeout" />
      </div>
    </Popup>
  </div>
</template>

<style>
  @import url('./assets/css/main.css');

  @font-face {
    font-family: 'dejavu';
    src: url('./assets/fonts/DejaVuSans.ttf');
    font-weight: normal;
  }

  @font-face {
    font-family: 'dejavu';
    src: url('./assets/fonts/DejaVuSans-Bold.ttf');
    font-weight: bold;
    font-style: normal;
  }

  :root {
    --main-bg: #3c434b;
    --secondary-bg: #1f252e;
    --alt-bg: #093d52;
    --app-font: 'dejavu';
  }

  #app {
    font-family: var(--app-font);
    background-color: var(--main-bg);

    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;

    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
  }

  .ml-1 {
    margin-left: 8px;
  }

  .router-active {
    background: var(--secondary-bg);
  }

  .main-view {
    position: fixed;
    top: calc(3vh + 38px);
    left: 112px;
    padding-top: 12px;
    width: calc(100% - 144px);
    height: calc(100% - 94px);
  }

  .slider {
    -webkit-appearance: none;
    appearance: none;
    width: 100%;
    height: 4px;
    background: #d3d3d3;
    outline: none;
    opacity: 0.7;
    -webkit-transition: 0.2s;
    transition: opacity 0.2s;
  }

  .slider:hover {
    opacity: 1;
  }

  .slider::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 10px;
    height: 10px;
    background: #00ffff;
    border-radius: 20%;
    cursor: pointer;
  }

  form {
    max-width: 420px;
    margin: 5px auto;
    text-align: left;
    color: white;
  }

  label {
    font-size: 90%;
    width: 100%;
  }
</style>

<style scoped>
  .item {
    padding-bottom: 8px;
    text-align: left;
  }

  .config-plus {
    margin: 0px 4px 0px 2px;
    text-align: left;
  }
</style>
