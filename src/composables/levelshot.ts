import { ref, onMounted } from 'vue'
import { convertFileSrc } from '@tauri-apps/api/core'
import { invoke } from '@tauri-apps/api/core'
import { ensureError } from '@/utils/util'
import { error } from '@tauri-apps/plugin-log'

const levelshots = ref<{ [key: string]: string } >({})
const loaded = ref(false)

export function useLevelshot() {

  async function getCachedLevelshots() {
    try {
      levelshots.value = await invoke('get_cached_levelshots')

      if (levelshots.value) {
        for (var key in levelshots.value) {
          levelshots.value[key] = convertFileSrc(levelshots.value[key]!)
        }
      }
    } catch (err) {
      error(`Error getting levelshots: ${ensureError(err).message}`)
    }
    
  }

  async function extractLevelshots(paths: string[]): Promise<number> {
    let extracted = 0
    try {
      extracted = await invoke('extract_levelshots_to_cache', { searchPaths: paths })
    } catch (err) {
      error(`Error extracting levelshots: ${ensureError(err).message}`)
    }
    return extracted
  }

  function levelHasLevelshot(levelName: string) {
    return levelName.toLowerCase() in levelshots.value
  }

  onMounted(async() => {
    if (!loaded.value) {
      loaded.value = true
      await getCachedLevelshots() 
    }
  })

  return {
    levelshots,
    levelHasLevelshot,
    getCachedLevelshots,
    extractLevelshots
  }
}
