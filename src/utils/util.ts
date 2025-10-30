import { type Config, type AppData } from '@/models/config'
import { type Quake3Server } from '@/models/server'
import { type Q3Executable } from '@/models/client'

export function ensureError(value: unknown): Error {
  if (value instanceof Error) return value

  let stringified = '[Unable to stringify the thrown value]'
  try {
    stringified = JSON.stringify(value)
  } catch {}

  const error = new Error(stringified)
  return error
}

export function defaultConfig(): Config {
  return {
    path: '',
    welcome_message: true,
    server_browser_threads: 50,
    server_timeout: 400,
    show_unreachable: false,
    manage_q3_instance: true,
    refresh_by_mod: false,
    show_trashed_servers: true,
    autoclose_demo: true,
    loop_demo: false,
    get_full_demo_data: true,
    q3_clients: [],
  }
}

export function defaultAppData(): AppData {
  return {
    path: '',
    pinned: new Set(),
    custom: new Set(),
    trash: new Set(),
    trash_ip: new Set(),
    masters: [],
    server_password: '',
  }
}

export function newCustomServer(ip_and_port: string[], address: string): Quake3Server {
  return {
    master: null,
    ip: ip_and_port[0]!,
    port: ip_and_port[1]!,
    address: address,
    protocol: null,
    ping: 0,
    errormessage: '',
    host: '',
    hostcolored: '',
    game: '',
    playersconnected: 0,
    maxclients: '0',
    bots: 0,
    map: '',
    othersettings: {},
    players: null,
    list: 'pinned',
    custom: true,
    version: '',
  }
}

export function validServerAddress(input: string): boolean {
  if (!input.includes(':') || !input.includes('.')) {
    return false
  }

  let split_s = input.split(':')
  let stripped = split_s[0]!.replaceAll('.', '')

  // port
  if (split_s.length > 2 || split_s[1]!.length == 0 || split_s[1]!.length > 5 || !/^\d+$/.test(split_s[1]!)) {
    return false
  }

  // ip/domain
  if (!/^[a-zA-Z0-9][a-zA-Z0-9-]+[a-zA-Z0-9]$/.test(stripped)) {
    return false
  }

  return true
}

export function validIp(input: string): boolean {
  let rx =
    /^(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$/ //

  if (!rx.test(input)) {
    return false
  }
  return true
}

export const Q3_BOT_NAMES: string[] = [
  'anarki',
  'angel',
  'biker',
  'bitterman',
  'bones',
  'cadavre',
  'crash',
  'daemia',
  'doom',
  'gorre',
  'grunt',
  'hossman',
  'hunter',
  'keel',
  'klesk',
  'lucy',
  'major',
  'mynx',
  'orbb',
  'patriot',
  'phobos',
  'ranger',
  'razor',
  'sarge',
  'slash',
  'sorlag',
  'stripe',
  'tankjr',
  'uriel',
  'visor',
  'wrack',
  'xaero',
]

export const UT_BOT_NAMES: string[] = [
  'boa',
  'cheetah',
  'chicken',
  'cobra',
  'cockroach',
  'cougar',
  'goose',
  'mantis',
  'penguin',
  'puma',
  'python',
  'raven',
  'scarab',
  'scorpion',
  'tiger',
  'widow',
]

export const CPMA_BOT_NAMES: string[] = [
  'anarki',
  'angel',
  'apheleon',
  'arQon',
  'biker',
  'bitterman',
  'bones',
  'cadavre',
  'crash',
  'daemia',
  'doom',
  'gorre',
  'grunt',
  'hossman',
  'hunter',
  'keel',
  'klesk',
  'lucy',
  'major',
  'mynx',
  'orbb',
  'patriot',
  'phobos',
  'ranger',
  'rat',
  'razor',
  'sarge',
  'slash',
  'sorlag',
  'stripe',
  'tankjr',
  'uriel',
  'visor',
  'wrack',
  'xaero',
]

export const OA_BOT_NAMES: string[] = [
  'angelyss',
  'andriy',
  'arachna',
  'assassin',
  'ayumi',
  'beret',
  'broadklin',
  'cyber-garg',
  'dark',
  'gargoyle',
  'ghost',
  'grism',
  'grunt',
  'headcrash',
  'jenna',
  'kyonshi',
  'liz',
  'major',
  'merman',
  'metalbot',
  'morgan',
  'murielle',
  'neko',
  'nekoyss',
  'penguin',
  'rai',
  's_marine',
  'sarge',
  'sergei',
  'skelebot',
  'sorceress',
  'tanisha',
  'tony',
]

export function getClientGameProtocol(client: Q3Executable | null): number {
  let protocol71Mods = ['baseoa', 'rat', 'missionpack']

  if (!client) { return 68}
  if (client.name.includes('liliumarenaclassic')) { return 43 }
  if (protocol71Mods.includes(client.gamename)) { return 71 }
  return 68
}

export function getServerProtocol(serv: Quake3Server): string {
  if (serv.protocol) {
    return serv.protocol.toString()
  }
  if (serv.othersettings.hasOwnProperty('protocol')) {
    return serv.othersettings['protocol']
  }
  return '68'
}

export async function getLatestGithubRelease() {
  const url = "https://api.github.com/repos/endless-r0ad/sarge-launcher/releases/latest"

  try {
    const response = await fetch(url)
    if (!response.ok) {
      throw new Error(`Request for latest release version status: ${response.status}`)
    }
    const result = await response.json()

    return result.tag_name
  } catch (err) {
    throw err
  }
}