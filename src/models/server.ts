import { type Nullable } from '@/utils/util'
import { type MasterServer } from '@/models/master'

export interface ServerPlayer {
  name: string
  namecolored: string
  frags: number
  ping: number
}

export interface Quake3Server {
  master: Nullable<MasterServer>
  ip: string
  port: string
  address: string
  protocol: Nullable<number>
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
  players: Nullable<ServerPlayer[]>
  list: string
  custom: boolean
  version: string
}
