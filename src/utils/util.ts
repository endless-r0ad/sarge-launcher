import { type Config, type AppData } from '@/models/config'
import { type Quake3Server } from '@/models/server'

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
    show_trashed_servers: true,
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
    ip: ip_and_port[0],
    port: ip_and_port[1],
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
  let stripped = split_s[0].replaceAll('.', '')

  // port
  if (split_s.length > 2 || split_s[1].length == 0 || split_s[1].length > 5 || !/^\d+$/.test(split_s[1])) {
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
  'biker',
  'bitterman',
  'bones',
  'crash',
  'doom',
  'grunt',
  'hunter',
  'keel',
  'klesk',
  'lucy',
  'major',
  'mynx',
  'orbb',
  'ranger',
  'razor',
  'sarge',
  'slash',
  'sorlag',
  'tankjr',
  'uriel',
  'visor',
  'xaero',
]
