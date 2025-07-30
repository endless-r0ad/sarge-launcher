import { type MasterServer } from '@/models/master'

export interface ServerPlayer {
  name: string
  namecolored: string
  frags: number
  ping: number
}

export interface Quake3Server {
  master: MasterServer | null
  ip: string
  port: string
  address: string
  protocol: number | null
  ping: number
  errormessage: string
  host: string
  hostcolored: string
  game: string
  playersconnected: number
  maxclients: string
  bots: number
  map: string
  othersettings: { [key: string]: string }
  players: ServerPlayer[] | null
  list: string
  custom: boolean
  version: string
}
