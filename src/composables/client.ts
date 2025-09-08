import { ref, onMounted, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { Q3Executable } from '@/models/client'
import { useConfig } from '@/composables/config'
import { error } from '@tauri-apps/plugin-log'
import { ensureError } from '@/utils/util'

const activeClient = ref<Q3Executable | null>(null)
const clientPaths = ref<string[]>([])

export function useClient() {

  const {
    config,
    writeConfig
  } = useConfig()

  const clientGame = computed(() => {
    if (!activeClient.value) {
      return null
    }
    return activeClient.value.gamename == 'q3ut4' ? 'q3urt4' : activeClient.value.gamename
  })

  async function getClientPaths(c: Q3Executable): Promise<string[]> {
    let paths: string[] = []
    try {
      paths = await invoke('get_client_paths', { activeClient: c })
      return paths
    } catch (err) {
      error(ensureError(err).message)
    }
    return paths
  }

  async function addQ3Client(client: Q3Executable) {
    if (config.value.q3_clients.length > 0) {
      config.value.q3_clients.map((client) => (client.active = false))
    }
    client.active = true
    config.value.q3_clients.push(client)
    clientPaths.value = await getClientPaths(client)

    activeClient.value = client
  }

  async function toggleQ3Client(client: Q3Executable) {
    if (client == activeClient.value) {
      return
    }
    config.value.q3_clients.map((c) => {
      c.active = c.exe_path == client.exe_path
    })
    clientPaths.value = await getClientPaths(client)

    activeClient.value = client
  }

  async function deleteQ3Client(client: Q3Executable) {
    if (config.value.q3_clients.length == 0) {
      return
    }

    config.value.q3_clients = config.value.q3_clients.filter((c) => c.exe_path != client.exe_path)
    if (config.value.q3_clients.length == 0) {
      activeClient.value = null
      clientPaths.value = []
    }
    if (config.value.q3_clients.length > 0 && !config.value.q3_clients.some((x) => x.active)) {
      config.value.q3_clients[0].active = true
      clientPaths.value = await getClientPaths(config.value.q3_clients[0])
      activeClient.value = config.value.q3_clients[0]
    }
  }

  async function pickClient(): Promise<boolean> {
    try {
      let new_client: Q3Executable = await invoke('pick_client')

      if (new_client != null) {
        if (config.value.q3_clients.some((c) => c.exe_path === new_client.exe_path)) {
          return false
        }
        addQ3Client(new_client)
        await writeConfig()
      }
      return true
    } catch (err) {
      throw err
    }
  }

  function clientAlreadyActivated(): boolean {
    return config.value.q3_clients.some((x) => x.active)
  }

  onMounted( async()=>{
    if (config.value.q3_clients.length > 0 && !clientAlreadyActivated()) {
      let client: Q3Executable = config.value.q3_clients[0]
      client.active = true
      clientPaths.value = await getClientPaths(client)
      activeClient.value = client
    }
  })

  return {
    activeClient,
    clientGame,
    toggleQ3Client,
    deleteQ3Client,
    pickClient,
    clientPaths
  }
}
