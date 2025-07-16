import { type Nullable } from '@/utils/util'
import { type Q3Executable } from '@/models/client'
import { type MasterServer } from '@/models/master'

export interface Config {
  path: string
  welcome_message: boolean
  play_gif: boolean
  server_browser_threads: number
  server_timeout: number
  show_unreachable: boolean
  manage_q3_instance: boolean
  show_trashed_servers: boolean
  demo_path: Nullable<string>
  fs_homepath: Nullable<string>
  q3_clients: Q3Executable[]
}

export interface AppData {
  path: string
  pinned: Set<string>
  custom: Set<string>
  trash: Set<string>
  trash_ip: Set<string>
  server_password: string
  masters: MasterServer[]
}
