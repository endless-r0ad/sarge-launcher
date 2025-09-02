import { onBeforeMount, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { Config } from '@/models/config'
import { ensureError, defaultConfig } from '@/utils/util'
import { error } from '@tauri-apps/plugin-log'

const config = ref<Config>(defaultConfig())
const loaded = ref(false)

export function useConfig() {

  async function writeConfig() {
    await invoke('save_config', { updatedConfig: config.value })
  }

  onBeforeMount( async()=>{
    if (!loaded.value) {
      try {
        config.value = await invoke('get_config')
      } catch(err) {
        error(`Error loading config: ${ensureError(err).message}`)
        config.value = defaultConfig()
      }
      loaded.value = true
    }
  })

  return {
    config,
    writeConfig
  }
}
