import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { sep } from '@tauri-apps/api/path'
import type { Demo } from '@/models/demo'
import { useConfig } from '@/composables/config'
import { useClient } from './client'
import { getServerProtocol, ensureError} from '@/utils/util'
import type { Quake3Server } from '@/models/server'
import type { Level } from '@/models/level'
import type { Bot } from '@/models/singleplayer'

const q3ClientProcessId = ref<number | null>(null)

export function useSpawnQuake() {

  const { config } = useConfig()
  const { activeClient, activeClientDefaultArgs, activeClientUserArgs } = useClient()

  async function spawnQuake(viewSuppliedArgs: string[]): Promise<string> {
    let ret = ''

    if (!activeClient.value) { return 'no client' }

    try {
      if (config.value.manage_q3_instance) {
        await invoke('kill_q3_client', { processId: q3ClientProcessId.value })
      }
      q3ClientProcessId.value = await invoke('spawn_client', {
        activeClient: activeClient.value,
        q3Args: activeClientDefaultArgs.value.concat(viewSuppliedArgs).concat(activeClientUserArgs.value)
      })
    } catch (err) {
      ret = ensureError(err).message
    }
    return ret
  }

  async function spawnQuakeServer(selectedServer: Quake3Server | null): Promise<string> {
    let ret = ''

    if (!selectedServer) { return ret }

    try {
      let args = ['+set', 'fs_game', selectedServer.game, '+set', 'protocol', getServerProtocol(selectedServer), '+connect', selectedServer.address];
      if ('g_needpass' in selectedServer.othersettings && selectedServer.othersettings['g_needpass'] == '1') {
        ret = 'needs password'
      } else {
        ret = await spawnQuake(args)
      }
    } catch(err) {
      ret = ensureError(err).message
    }
    return ret 
  }

  async function spawnQuakeDemo(selectedDemo: Demo | null): Promise<string> {
    let ret = ''

    if (!selectedDemo) { return ret }

    try {
      let args = ['+set', 'fs_game', selectedDemo.gamename, '+exec', 'sarge-launcher-demo.cfg']
      let demos_index = selectedDemo.path.indexOf(sep() + 'demos')
      let relative_path = selectedDemo.path.substring(demos_index + 7)
      let path = relative_path.substring(0, relative_path.lastIndexOf('.'))
  
      await invoke('create_demo_script', {
        activeClient: activeClient.value, 
        fsGame: selectedDemo.gamename, 
        demoPath: path, 
        close: config.value.autoclose_demo, 
        loopD: config.value.loop_demo
      })

      ret = await spawnQuake(args)
    } catch (err) {
      ret = ensureError(err).message
    }
    return ret
  }

  async function spawnQuakeDefrag(selectedLevel: Level | null, game: string, cheats: boolean, overbounces: boolean): Promise<string> {
    let ret = ''

    if (!selectedLevel) { return ret }

    let args = [cheats ? '+dev' + game : '+' + game, selectedLevel.level_name]

    args.push(...['+set', 'df_ob_KillOBs', overbounces ? '0' : '1'])
    args.push(...['+set', 'sv_maxclients', '8'])
    args.push(...['+set', 'skill', '3', '+wait', '3'])
    args.push(...['+wait', '5'])
    args.push(...['+team', 'Free'])

    ret = await spawnQuake(args)

    return ret
  }

  async function spawnQuakeSinglePlayer(selectedLevel: Level | null, 
                                        gameType: number, 
                                        gametypes: string[], 
                                        cheats: boolean, 
                                        sv_maxclients: number,
                                        difficulty: number,
                                        teamFreeBotsAllowed: boolean,
                                        isTeamGameType: boolean,
                                        teamSelect: 'Free' | 'Red' | 'Blue',
                                        bots_team_free: Bot[],
                                        bots_team_red: Bot[],
                                        bots_team_blue: Bot[]): Promise<string> {
    let ret = ''

    try {
      if (!selectedLevel) { return ret }

      let gametypeName = gametypes[gameType] ?? ''
      let args = []
      let launch = ''

      if (gametypeName == 'SP') { launch = cheats ? '+spdevmap' : '+spmap' } 
      else { launch = cheats ? '+devmap' : '+map' }

      args = [launch, selectedLevel.level_name]

      if (activeClient.value?.gamename == 'cpma') {
        args.push(...['+set', 'server_useMapModes' , '0', '+set', 'mode_start', gametypeName])
      } else {
        args.push(...['+set', 'g_gametype', gameType.toString()])
      }

      args.push(...['+set', 'sv_maxclients', sv_maxclients.toString()])
      args.push(...['+set', gametypeName == 'SP' ? 'g_spskill' : 'skill', difficulty.toString(), '+wait', '3'])

      if (activeClient.value?.gamename == 'q3ut4' && gametypeName == 'JUMP') {
        args.push(...['+set', 'g_stamina', '2'])
        args.push(...['+set', 'g_allowPosSaving', '1'])
        args.push(...['+set', 'g_allowGoto', '1'])
        args.push(...['+set', 'g_novest', '1'])
        args.push(...['+set', 'g_warmup', '0'])
        args.push(...['+set', 'g_nodamage', '1'])
        args.push(...['+set', 'g_respawndelay', '0'])
      }

      if (teamFreeBotsAllowed) {
        if (bots_team_free.length && activeClient.value?.gamename == 'q3ut4') {
          args.push(...['+set', 'bot_enable', '1'])
        }
        bots_team_free.forEach((bot) => {
          args.push(...['+addbot', bot.name, bot.difficulty.toString()])
        })
      }

      if (isTeamGameType) {
        if ((bots_team_red.length || bots_team_blue.length) && activeClient.value?.gamename == 'q3ut4') {
          args.push(...['+set', 'bot_enable', '1'])
        }
        bots_team_red.forEach((bot) => {
          args.push(...['+addbot', bot.name, bot.difficulty.toString(), bot.team])
        })

        bots_team_blue.forEach((bot) => {
          args.push(...['+addbot', bot.name, bot.difficulty.toString(), bot.team])
        })
      }

      args.push(...['+wait', '5'])
      args.push(...['+team', teamSelect])

      ret = await spawnQuake(args)
    } catch(err) {
      ret = ensureError(err).message
    }
    return ret
  }

  return {
    spawnQuake,
    spawnQuakeDemo,
    spawnQuakeServer,
    spawnQuakeSinglePlayer,
    spawnQuakeDefrag
  }
}
