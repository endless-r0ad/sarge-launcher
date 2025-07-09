import { ref } from 'vue'
import { type Nullable } from '@/utils/util'
import { convertFileSrc } from '@tauri-apps/api/core'
import { type Level } from '@/models/level'
import { invoke } from '@tauri-apps/api/core'

export function useLevelshot() {
  const levelshots = ref<Nullable<{ [key: string]: string }>>(null)
  const levels = ref<Level[]>([])

  async function getLevels(levelPath: Nullable<string>) {
    levels.value = await invoke('get_levels', { fsHomepath: levelPath })
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

  async function syncLevelshots(levelPath: Nullable<string>) {
    await getLevels(levelPath)
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
