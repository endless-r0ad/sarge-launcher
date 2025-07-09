import { type Nullable } from '@/utils/util'

export interface DemoPlayer {
  name: string
  namecolored: string
}

export interface Demo {
  file_name: string
  path: string
  protocol: number
  gamename: string
  mapname: string
  g_gametype: string
  player_pov: DemoPlayer
  snapshots: number
  sv_hostname: string
  sv_hostname_color: string
  players: Nullable<DemoPlayer[]>
  server_info: { [key: string]: string }
  system_info: { [key: string]: string }
  server_commands: { [key: number]: string }
  duration: number
  df_time: number
  issue: Nullable<string>
  version: string
}
