import { onBeforeMount, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { Q3Executable } from '@/models/client'
import type { Config } from '@/models/config'
import { ensureError, defaultConfig } from '@/utils/util'
import { error } from '@tauri-apps/plugin-log'

const config = ref<Config>(defaultConfig())
const activeClient = ref<Q3Executable | null>(null)
const loaded = ref(false)

export function useConfig() {

  function addQ3Client(client: Q3Executable) {
    if (config.value.q3_clients.length > 0) {
      config.value.q3_clients.map((client) => (client.active = false))
    }
    client.active = true
    config.value.q3_clients.push(client)
    activeClient.value = client
  }

  function toggleQ3Client(client: Q3Executable) {
    if (client == activeClient.value) {
      return
    }
    config.value.q3_clients.map((c) => {
      c.active = c.exe_path == client.exe_path
    })
    activeClient.value = client
  }

  function deleteQ3Client(client: Q3Executable) {
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
  }

  async function writeConfig() {
    await invoke('save_config', { updatedConfig: config.value })
  }

  onBeforeMount( async()=>{
    if (!loaded.value) {
      try {
        config.value = await invoke('get_config')
        if (config.value.q3_clients.length > 0) {
          let client: Q3Executable = config.value.q3_clients[0]
          client.active = true
          activeClient.value = client
        }
      } catch(err) {
        error(`Error loading config: ${ensureError(err).message}`)
        config.value = defaultConfig()
      }
      loaded.value = true
    }
  })

  return {
    config,
    activeClient,
    addQ3Client,
    toggleQ3Client,
    deleteQ3Client,
    writeConfig
  }
}
