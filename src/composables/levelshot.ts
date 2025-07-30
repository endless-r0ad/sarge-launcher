import { ref } from 'vue'
import { convertFileSrc } from '@tauri-apps/api/core'
import { type Level } from '@/models/level'
import { invoke } from '@tauri-apps/api/core'
import type { Q3Executable } from '@/models/client'

export function useLevelshot() {
  const levelshots = ref<{ [key: string]: string } | null>(null)
  const levels = ref<Level[]>([])
  const activeClientPaths = ref<string[]>([])

  async function getClientPaths(activeClient: Q3Executable | null) {
    console.log('getClientPaths from ', activeClient)
    activeClientPaths.value = await invoke('get_client_paths', { activeClient: activeClient })
  }
  
  async function getLevels(getAllData: boolean) {
    levels.value = await invoke('get_levels', { searchPaths: activeClientPaths.value, getAllData: getAllData })
  }

  async function extractLevelshotsToCache() {
    await invoke('extract_levelshots_to_cache', { levels: levels.value })
  }

  async function getCachedLevelshots() {
    levelshots.value = await invoke('get_cached_levelshots')

    if (levelshots.value) {
      for (var key in levelshots.value) {
        levelshots.value[key] = convertFileSrc(levelshots.value[key])
      }
    }
  }

  async function syncLevelshots(activeClient: Q3Executable | null, getAllData: boolean) {
    await getClientPaths(activeClient)
    await getLevels(getAllData)
    await extractLevelshotsToCache()
    await getCachedLevelshots()
  }

  function levelHasLevelshot(levelName: string) {
    if (levelshots.value) {
      if (levelName.toLowerCase() in levelshots.value) {
        return true
      }
    }
    return false
  }

  return {
    levels,
    levelshots,
    levelHasLevelshot,
    syncLevelshots,
  }
}
