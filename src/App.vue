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

  const { config } = useConfig()
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
  
  const currentComponent = ref('')

  function getComponentName(componentName: string) { 
    currentComponent.value = componentName 
  }

  const showSettings = ref(false)

  const alertMessage = ref('')
  const alertType = ref('')

  function alert(type: string, msg: string) {
    alertType.value = type
    type == 'info' ? info(msg) : error(msg)
    alertMessage.value = msg
  }

  const q3ClientProcessId = ref<number | null>(null)

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
        alert('info', 'Please configure a Quake 3 client to launch')
      } else {
        alert('error', e.message)
      }
    }
  }

</script>

<template>
  <Loading v-if="!isMounted" :position="'center'" :message="'loading...'" :size="90" />
  <Header v-if="isMounted" :currentView="currentComponent" @spawnQuake="spawnQuake" @alert="alert" />
  <Sidebar v-if="isMounted" :showSettings="showSettings" @toggleSettings="showSettings=!showSettings" @exitApp="invoke('exit_app')" />

  <main v-if="isMounted" class="main-view">
    <router-view v-slot="{ Component }">
      <KeepAlive>
        <component
          :is="Component"
          :latestGithubVersion="latestRelease"
          @spawnQuake="spawnQuake"
          @emitComponentName="getComponentName"
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
