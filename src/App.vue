<script setup lang="ts">
  import Loading from '@/components/Loading.vue'
  import Header from '@/components/Header.vue'
  import Modal from '@/components/Modal.vue'
  import Sidebar from '@/components/Sidebar.vue'
  import Settings from '@/components/Settings.vue'
  import { ensureError, getLatestGithubRelease } from '@/utils/util'
  import { ref, onMounted } from 'vue'
  import { invoke } from '@tauri-apps/api/core'
  import { info, error } from '@tauri-apps/plugin-log'
  import { useConfig } from './composables/config'
  import { useClient } from './composables/client'

  const { config, writeConfig } = useConfig()
  const { activeClient, activeClientDefaultArgs, activeClientUserArgs } = useClient()

  const isMounted = ref(false)
  const latestRelease = ref<string | null>(null)

  onMounted(async () => {
    try {
      latestRelease.value = await getLatestGithubRelease()
    } catch (err) {
      error(ensureError(err).message)
    }
    await new Promise((r) => setTimeout(r, 300))
    isMounted.value = true
  })

  async function saveConfig() {
    try {
      showSettings.value = false
      await writeConfig()
    } catch (err) {
      errorAlert(ensureError(err).message)
    }
  }
  
  const q3ClientProcessId = ref<number | null>(null)
  const currentComponent = ref('')

  function getComponentName(componentName: string) { 
    currentComponent.value = componentName 
  }

  const showSettings = ref(false)

  function toggleSettings() { showSettings.value = !showSettings.value }

  const showAlertPopup = ref(false)
  const popupType = ref('')
  const alertMessage = ref('')

  function infoAlert(infoMsg: string) {
    info(infoMsg)
    showAlertPopup.value = true 
    popupType.value = 'info'
    alertMessage.value = infoMsg
  }

  function errorAlert(err: string) {
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

  async function spawnQuake(viewSuppliedArgs: string[]) {
    try {
      if (config.value.manage_q3_instance) {
        await invoke('kill_q3_client', { processId: q3ClientProcessId.value })
      }
      q3ClientProcessId.value = await invoke('spawn_client', {
        activeClient: activeClient.value,
        q3Args: activeClientDefaultArgs.value.concat(viewSuppliedArgs).concat(activeClientUserArgs.value)
      })
    } catch (err) {
      const e: Error = ensureError(err)
      if (e.message.includes('expected struct Q3Executable') || e.message.includes('missing required key activeClient')) {
        infoAlert('Please configure a Quake 3 client to launch')
      } else {
        errorAlert(e.message)
      }
    }
  }

</script>

<template>
  <Loading v-if="!isMounted" :position="'center'" :message="'loading...'" :size="90" />

  <Header
    v-if="isMounted"
    :currentView="currentComponent"
    @spawnQuake="spawnQuake"
    @errorAlert="errorAlert"
    @infoAlert="infoAlert"
  />

  <Sidebar v-if="isMounted" :showSettings="showSettings" @toggleSettings="toggleSettings" @exitApp="invoke('exit_app')" />

  <main v-if="isMounted" class="main-view">
    <router-view v-slot="{ Component }">
      <KeepAlive>
        <component
          :is="Component"
          :latestGithubVersion="latestRelease"
          @spawnQuake="spawnQuake"
          @emitComponentName="getComponentName"
          @errorAlert="errorAlert"
          @infoAlert="infoAlert"
        />
      </KeepAlive>
    </router-view>
  </main>

  <div id="modal">
    <Modal v-if="showAlertPopup" :popupType="popupType" @cancelModal="closeAlert" @executeModal="closeAlert">
      {{ alertMessage }}
    </Modal>
    
    <Modal v-if="showSettings" :popupType="'center'" @executeModal="saveConfig" @cancelModal="saveConfig">
      <Settings />
    </Modal>
  </div>
</template>

<style>
  @import url('./assets/css/shared.css');

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

  .main-view {
    position: fixed;
    top: calc(3vh + 38px);
    left: 112px;
    padding-top: 12px;
    width: calc(100% - 144px);
    height: calc(100% - 94px);
  }

</style>
