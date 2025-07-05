<script setup lang="ts">

  import Loading from '@/components/Loading.vue'
  import Header from '@/components/Header.vue';
  import Popup from '@/components/Popup.vue';
  import Sidebar from '@/components/Sidebar.vue';
  import { type Nullable, ensureError, tempConfig, tempAppData } from '@/utils/util';
  import { ref, onMounted } from 'vue';
  import { invoke } from '@tauri-apps/api/core'
  import { info, error } from '@tauri-apps/plugin-log';
  import { type Config, type AppData } from '@/models/config';
  import { type Q3Executable } from '@/models/client';

  const config = ref<Config>( tempConfig() );
  const appData = ref<AppData>( tempAppData() );

  onMounted(async () => {
    try {
      config.value = await invoke('get_config');
      appData.value = await invoke('get_appdata');
      appData.value.pinned = new Set(appData.value.pinned)
      appData.value.custom = new Set(appData.value.custom)
      appData.value.trash = new Set(appData.value.trash)
      appData.value.trash_ip = new Set(appData.value.trash_ip)

      activeClient.value = config.value?.q3_clients?.filter(c => c.active)[0] as Q3Executable

      await new Promise(r => setTimeout(r, 800)); // show the loading animation for a bit
      isMounted.value = true;
    }
    catch(err) {
      showErrorAlert(ensureError(err).message)
    }    
  })

  async function saveConfig() {  
    try {
      showSettings.value = false 
      await invoke('save_config', { updatedConfig: config.value })
    }
    catch(err) {
      showErrorAlert(ensureError(err).message)
    }
  }

  async function mutateConfig(newConfig: Config) {
    config.value = newConfig
    await saveConfig();
  }

  async function mutateAppData(newAppData: AppData) {
    appData.value = newAppData

    const appDataForBackend = {
      ...newAppData,
      pinned: Array.from(newAppData.pinned), 
      custom: Array.from(newAppData.custom), 
      trash: Array.from(newAppData.trash),
      trash_ip: Array.from(newAppData.trash_ip)
    }
    await invoke('save_app_data', { appData: appDataForBackend})
  }

  const isMounted = ref(false);
  const q3ClientIsOpen = ref(false);
  const q3ClientProcessId = ref<Nullable<number>>(null);
  const currentComponent = ref('ad');

  function getComponentName(componentName: string) { currentComponent.value = componentName }

  const showSettings = ref(false);

  function toggleSettings() { showSettings.value = !showSettings.value }

  const spawnArgs = ref<string[]>([]);

  function getConnectArgs(args: string[]) { spawnArgs.value = args }

  const showAlertPopup = ref(false);
  const popupType = ref('');
  const alertMessage = ref('');

  function showInfoAlert(infoMsg: string) {
    info(infoMsg)
    showAlertPopup.value = true, popupType.value = 'info', alertMessage.value = infoMsg
  }

  function showErrorAlert(err: string) {
    error(err)
    showAlertPopup.value = true, popupType.value = 'error', alertMessage.value = err
  }

  function closeAlert() { showAlertPopup.value = false, popupType.value = '', alertMessage.value = '' }

  const activeClient = ref<Nullable<Q3Executable>>(null)

  async function addQ3Client(client: Q3Executable) {
    if  (config.value && config.value.q3_clients != null) {
      if (config.value.q3_clients.length > 0) {
        if (config.value.q3_clients.some((c) => c.path === client.path )) {
          showInfoAlert('client already added')
        } else {
          config.value.q3_clients.map((client) => client.active = false);
          let newClient: Q3Executable = {name: client.name, path: client.path, active: true};
          config.value.q3_clients.push(newClient);
          activeClient.value = newClient
          await saveConfig()
        }    
      } else {
        let newClient: Q3Executable = {name: client.name, path: client.path, active: true};
        config.value.q3_clients.push(newClient);
        activeClient.value = newClient
        await saveConfig()
      } 
    } 
  }

  async function toggleQ3Client(client: Q3Executable) {
    if (client == activeClient.value) { return }
    config.value?.q3_clients?.map((c) => { c.active = c.path == client.path });
    activeClient.value = client
    await saveConfig();
  }

  async function deleteQ3Client(client: Q3Executable) {
    if  (config.value && config.value.q3_clients) {
      if (config.value.q3_clients.length == 0) { return }
      config.value.q3_clients = config.value.q3_clients.filter((c) => c.path != client.path)
      if (config.value.q3_clients.length == 0) {
        activeClient.value = null
      }
      if (config.value.q3_clients.length > 0 && !config.value.q3_clients.some((x) => x.active)){
        config.value.q3_clients[0].active = true
        activeClient.value = config.value.q3_clients[0]
      }
      await saveConfig();
    }
  
  }
  
  async function spawnQuake(source: string) {
    if (config.value?.manage_q3_instance && q3ClientProcessId.value !== null) {
      try {
        await invoke('kill_q3_client', {processId: q3ClientProcessId.value})
      }
      catch(err) {
        showErrorAlert(ensureError(err).message)
      }
    }

    let id = 0;

    try {
      id = await invoke('spawn_quake', {
        activeClient: activeClient.value, 
        q3Args: source == 'Header' ? [] : spawnArgs.value, 
        manageInstance: config.value?.manage_q3_instance
      })

      q3ClientProcessId.value = id;
      q3ClientIsOpen.value = true;
    }
    catch(err) {
      const e: Error = ensureError(err)
      if (e.message.includes('expected struct Q3Executable') || e.message.includes('missing required key activeClient')) {
        showInfoAlert('Please configure a Quake 3 client to launch')
      } else {
        showErrorAlert(e.message)
      }
    }
    q3ClientIsOpen.value = false; // maybe actually keep track? 
  }

  async function exitApp() { await invoke('exit_app') }

</script>

<template>

  <Loading v-if="!isMounted" :position="'center'" :message="'loading...'" :size="90"/>

  <Header v-if="isMounted"
          :currentView="currentComponent"
          :config="config"
          :activeClient="activeClient"
          @spawnQuake="spawnQuake"
          @addQ3Client="addQ3Client"
          @toggleQ3Client="toggleQ3Client"
          @deleteQ3Client="deleteQ3Client"
          @errorAlert="showErrorAlert" />  

  <Sidebar v-if="isMounted" 
          :showSettings="showSettings" 
          @toggleSettings="toggleSettings" 
          @exitApp="exitApp"/>

  <div v-if="isMounted" class="main-view">
      <router-view v-slot="{ Component }">
        <KeepAlive exclude="Home">
          <component :is="Component"
                  :config="config"
                  :appData="appData"
                  :showUnreachableServers="config?.show_unreachable"
                  :showTrashedServers="config?.show_trashed_servers"
                  @mutateConfig="mutateConfig"
                  @mutateAppData="mutateAppData"
                  @spawnQuake="spawnQuake"
                  @addQ3Client="addQ3Client"
                  @emitConnectArgs="getConnectArgs"
                  @emitComponentName="getComponentName" 
                  @errorAlert="showErrorAlert"
                  @infoAlert="showInfoAlert" />
        </KeepAlive>
      </router-view>                   
  </div>  
  
  <div id="popup">
    <!-- <Transition name="pop"> -->
      <Popup v-if="showAlertPopup" :popupType="popupType" @cancelModal="closeAlert" @executeModal="closeAlert">{{ alertMessage }}</Popup>
    <!-- </Transition> -->
    <!-- <Transition name="pop"> -->
      <Popup v-if="showSettings" :popupType="'center'" @executeModal="saveConfig" @cancelModal="saveConfig">
          <div class="item"><input type="checkbox" v-model="config.manage_q3_instance"><label class="ml-1">Manage Q3 Instance</label></div>    
          <div class="item"><input type="checkbox" v-model="config.show_unreachable"><label class="ml-1">Show Unreachable Servers</label></div> 
          <div class="item"><input type="checkbox" v-model="config.show_trashed_servers"><label class="ml-1">Show Trashed Servers</label></div> 

          <div class="config-plus">+<label class="ml-1">Browser Threads - {{ config.server_browser_threads == 0 ? 1 : config.server_browser_threads  }}</label></div>
          <div class="item"><input type="range" min="0" max="120" step="5" value="60" class="slider" v-model.number="config.server_browser_threads"></div> 

          <div class="config-plus">+<label class="ml-1">Server Timeout - {{ config.server_timeout }}ms</label></div>
          <div class="item"><input type="range" min="200" max="1000" step="100" value="400" class="slider" v-model.number="config.server_timeout"></div>  
      </Popup>
    <!-- </Transition> -->
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

  .router-active{
    background: var(--secondary-bg);
  }

  .main-view {
    position: fixed;
    top: calc(3vh + 38px);  
    left: 112px; /* sidebar */
    /* padding: 0 10px 10px 10px;   */
    padding-top: 12px;
    width: calc(100% - 144px);
    height: calc(100% - 94px);
    /* top: 54px; */
    /* bottom: 0; */
  }

  /* The switch - the box around the slider */
  .switch {
    position: relative;
    display: inline-block;
    width: 60px;
    height: 34px;
  }

  /* Hide default HTML checkbox */
  .switch input {
    opacity: 0;
    width: 0;
    height: 0;
  }


  /* The slider itself */
  .slider {
    -webkit-appearance: none;  /* Override default CSS styles */
    appearance: none;
    width: 100%; /* Full-width */
    height: 4px; /* Specified height */
    background: #d3d3d3; /* Grey background */
    outline: none; /* Remove outline */
    opacity: 0.7; /* Set transparency (for mouse-over effects on hover) */
    -webkit-transition: .2s; /* 0.2 seconds transition on hover */
    transition: opacity .2s;
  }

  /* Mouse-over effects */
  .slider:hover {
    opacity: 1; /* Fully shown on mouse-over */
  }

  /* The slider handle (use -webkit- (Chrome, Opera, Safari, Edge) and -moz- (Firefox) to override default look) */
  .slider::-webkit-slider-thumb {
    -webkit-appearance: none; /* Override default look */
    appearance: none;
    width: 10px; /* Set a specific slider handle width */
    height: 10px; /* Slider handle height */
    background: #00ffff; /* Green background */
    border-radius: 20%;
    cursor: pointer; /* Cursor on hover */
  }

  .slider::-moz-range-thumb {
    width: 8px; /* Set a specific slider handle width */
    height: 8px; /* Slider handle height */
    background: #04AA6D; /* Green background */
    cursor: pointer; /* Cursor on hover */
  }

  form {
    max-width: 420px;
    margin: 5px auto;
    /* min-width: max-content; */
    /* background: white; */
    text-align: left;
    /* padding: 16px; */  
    /* background: rgba(0, 0, 0, 0.5); */
    color: white;
  }

  label {
    font-size: 90%;
    /* text-transform: uppercase; */
    /* letter-spacing: 1px;
    font-weight: bold; */
    width: 100%;
  }

</style>

<style scoped >
  .item {
    padding-bottom: 8px;
    text-align: left;
  }

  .config-plus {
    margin: 0px 4px 0px 2px; 
    text-align: left;
  }

  .pop-enter-from {
    opacity: 0.5;
    transform: scale(1.2);
  }
    
  .pop-leave-to {
    opacity: 0;
    transform: scale(1.2);
  }

  .pop-enter-to {
    opacity: 1;
    transform: scale(1);
  }

  .pop-enter-active,
  .pop-leave-active {
    transition: all 0.15s ease;
  }

  .pop-move {
    transition: all 0.15s ease;
  }


</style>