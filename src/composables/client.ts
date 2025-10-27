import { ref, onMounted, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { Q3Executable, Q3Config } from '@/models/client'
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
    config.value.q3_clients.map((c) => {
      c.active = c.exe_path == client.exe_path
    })
    clientPaths.value = await getClientPaths(client)

    activeClient.value = client
  }

  async function deleteQ3Client(client: Q3Executable) {
    if (config.value.q3_clients.length == 0 || clientConfigIndex(client) == -1)  {
      return
    }

    config.value.q3_clients = config.value.q3_clients.filter((c) => c.exe_path != client.exe_path)
    if (config.value.q3_clients.length == 0) {
      activeClient.value = null
      clientPaths.value = []
    }
    if (config.value.q3_clients.length > 0 && !config.value.q3_clients.some((x) => x.active)) {
      config.value.q3_clients[0]!.active = true
      clientPaths.value = await getClientPaths(config.value.q3_clients[0]!)
      activeClient.value = config.value.q3_clients[0]!
    }
    await writeConfig()
  }

  async function pickClient(): Promise<boolean> {
    try {
      let new_client: Q3Executable = await invoke('pick_client')

      if (new_client != null) {
        if (config.value.q3_clients.some((c) => c.exe_path === new_client.exe_path)) {
          return false
        }
        new_client.gamename = getClientDefaultGamename(new_client)
        addQ3Client(new_client)
        await writeConfig()
      }
      return true
    } catch (err) {
      throw err
    }
  }

  async function updateClient(client: Q3Executable) {
    try {
      let index = clientConfigIndex(client)
      if (index != -1) {
        config.value.q3_clients[index] = client
        if (client.exe_path === activeClient.value?.exe_path) {
          await toggleQ3Client(client)
        }
        await writeConfig()
      } 
    } catch (err) {
      error(ensureError(err).message)
    }
  }

  async function getClientConfigs(client: Q3Executable): Promise<Q3Config[]> {
    try {
      let paths = await getClientPaths(client)
      let q3Configs: Q3Config[] = await invoke('get_client_configs', {searchPaths: paths})
      return q3Configs
    } catch (err) {
      error(ensureError(err).message)
      return []
    }
  }

  function getClientGameProtocol(client: Q3Executable): number {
    if (client.name.includes('liliumarenaclassic')) { return 43 }
    if (client.gamename == 'baseoa') { return 71 }
    return 68
  }

  function getClientDefaultGamename(client: Q3Executable): string {
    switch (client.name.toLowerCase()) {
			case "quake3-urt":
      case "quake3e_urt.x64":
        return 'q3ut4'
			case "odfe":
      case "odfe.vk":
      case "odfe.x64":
      case "odfe.vk.x64":
      case 'idfe':
      case 'idfe.vk':
      case 'idfe.x64':
      case 'idfe.vk.x64':
      case 'idfe.x86_64':
      case 'idfe.vk.x86_64':
        return 'defrag'
      case 'cnq3-x64':
      case 'cnq3-x86':
        return 'cpma'
      case 'openarena':
      case 'omega-x64':
        return 'baseoa'
      default:
        return 'baseq3'
		}
  }

  function clientAlreadyActivated(): boolean {
    return config.value.q3_clients.some((x) => x.active)
  }

  function clientConfigIndex(client: Q3Executable): number {
    return config.value.q3_clients.findIndex((c) => c.exe_path === client.exe_path)
  }

  onMounted( async()=>{
    if (config.value.q3_clients.length > 0 && !clientAlreadyActivated()) {
      let client: Q3Executable = config.value.q3_clients[0]!
      client.active = true
      clientPaths.value = await getClientPaths(client)
      activeClient.value = client
    }
  })

  return {
    activeClient,
    clientGame,
    getClientGameProtocol,
    getClientDefaultGamename,
    updateClient,
    toggleQ3Client,
    deleteQ3Client,
    pickClient,
    clientPaths,
    getClientConfigs
  }
}
