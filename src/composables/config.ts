import { onMounted, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { Config } from '@/models/config'
import { ensureError, defaultConfig } from '@/utils/util'
import { error } from '@tauri-apps/plugin-log'

const config = ref<Config>(defaultConfig())
const loaded = ref(false)

watch(() => config.value.loop_demo, (newVal, _oldVal) => {
    if (newVal) {
      config.value.autoclose_demo = false
    }
  })

watch(() => config.value.autoclose_demo, (newVal, _oldVal) => {
  if (newVal) {
    config.value.loop_demo = false
  }
})

export function useConfig() {

  async function writeConfig() {
    await invoke('save_config', { updatedConfig: config.value })
  }

  onMounted( async()=>{
    if (!loaded.value) {
      try {
        loaded.value = true
        config.value = await invoke('get_config')
      } catch(err) {
        error(`Error loading config: ${ensureError(err).message}`)
        config.value = defaultConfig()
      }
      
    }
  })

  return {
    config,
    writeConfig
  }
}
