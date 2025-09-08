import { onMounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { AppData } from '@/models/config'
import { ensureError, defaultAppData } from '@/utils/util'
import { error } from '@tauri-apps/plugin-log'


const appdata = ref<AppData>(defaultAppData())
const loaded = ref(false)

export function useAppData() {

  function addAppData(whatList: string, address: string) {
    switch (whatList) {
      case 'custom':
        appdata.value.custom.add(address)
        break
      case 'pinned':
        appdata.value.pinned.add(address)
        break
      case 'trash':
        appdata.value.trash.add(address)
        break
      case 'trash_ip':
        appdata.value.trash_ip.add(address)
        break  
    }
  }

  function removeAppData(whatList: string, address: string) {
    switch (whatList) {
      case 'custom':
        appdata.value.custom.delete(address)
        break
      case 'pinned':
        appdata.value.pinned.delete(address)
        break
      case 'trash':
        appdata.value.trash.delete(address)
        break
      case 'trash_ip':
        appdata.value.trash_ip.delete(address)
        break  
    }
  }

  async function writeAppData() {

    const appDataForBackend = {
      ...appdata.value,
      pinned: Array.from(appdata.value.pinned),
      custom: Array.from(appdata.value.custom),
      trash: Array.from(appdata.value.trash),
      trash_ip: Array.from(appdata.value.trash_ip),
    }
    
    await invoke('save_app_data', { updatedData: appDataForBackend })
  }

  onMounted( async() => {
    if (!loaded.value) {
      try {
        loaded.value = true
        appdata.value = await invoke('get_appdata')
        appdata.value.pinned = new Set(appdata.value.pinned)
        appdata.value.custom = new Set(appdata.value.custom)
        appdata.value.trash = new Set(appdata.value.trash)
        appdata.value.trash_ip = new Set(appdata.value.trash_ip)
      } catch(err) {
        error(`Error loading appdata: ${ensureError(err).message}`)
        appdata.value = defaultAppData()
      }
    }
    
  })

  return {
    appdata,
    addAppData,
    removeAppData,
    writeAppData
  }
}
