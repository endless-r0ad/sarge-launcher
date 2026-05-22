<script setup lang="ts">
  import Loading from '@/components/Loading.vue'
  import Header from '@/components/Header.vue'
  import Modal from '@/components/Modal.vue'
  import Sidebar from '@/components/Sidebar.vue'
  import Settings from '@/components/Settings.vue'
  import SargeStatus from './components/SargeStatus.vue'
  import { ensureError, getLatestGithubRelease } from '@/utils/util'
  import { ref, onMounted } from 'vue'
  import { invoke } from '@tauri-apps/api/core'
  import { info, error } from '@tauri-apps/plugin-log'
  import { useConfig } from './composables/config'
  import { useAppData } from './composables/appdata'

  useAppData()
  const { config } = useConfig()

  const appVersion = 'v0.3.1'
  const isMounted = ref(false)
  const latestRelease = ref<string | null>(null)
  const updateAvailable = ref(false)
  const showUpdate = ref(false)
  const currentComponent = ref('')
  const showSettings = ref(false)

  onMounted(async () => {
    try {
      latestRelease.value = await getLatestGithubRelease()
      updateAvailable.value = latestRelease.value != null && appVersion != latestRelease.value
      showUpdate.value = updateAvailable.value
    } catch (err) {
      error(ensureError(err).message)
    }
    await new Promise((r) => setTimeout(r, 300))
    isMounted.value = true
  })
  
  const alertMessage = ref('')
  const alertType = ref('')

  function alert(type: string, msg: string) {
    alertType.value = type
    type == 'info' ? info(msg) : error(msg)
    alertMessage.value = msg
  }

</script>

<template>
  <Loading v-if="!isMounted" :position="'center'" :message="'loading...'" :size="90" />
  <Header v-if="isMounted" :currentView="currentComponent" @alert="alert" />
  <Sidebar v-if="isMounted" :showSettings="showSettings" @toggleSettings="showSettings=!showSettings" @exitApp="invoke('exit_app')" />

  <main v-if="isMounted" class="main-view">
    <router-view v-slot="{ Component }">
      <KeepAlive>
        <component
          :is="Component"
          @emitComponentName="currentComponent = $event"
          @alert="alert"
        />
      </KeepAlive>
    </router-view>
  </main>

  <div id="modal">
    <Modal v-if="alertMessage != ''" :popupType="alertType" @close="alertType = ''; alertMessage = '' ">
      {{ alertMessage }}
    </Modal>
    
    <Modal v-if="showSettings" :popupType="'center'" @close="showSettings=false">
      <Settings />
    </Modal>

    <Modal v-if="isMounted && (showUpdate || config.welcome_message)" 
          :popupType="'center'" 
          @close="config.welcome_message = false; showUpdate = false">
      <SargeStatus :updateAvailable="updateAvailable" 
                   :welcomeMessage="config.welcome_message" 
                   :appVersion="appVersion" 
                   :latestRelease="latestRelease" 
      />
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
