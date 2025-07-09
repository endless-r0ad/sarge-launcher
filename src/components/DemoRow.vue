<script setup lang="ts">
  import { watch, defineProps, defineEmits, ref, computed, onMounted, onUnmounted } from 'vue'
  import { type Demo } from '@/models/demo'
  import { type Nullable } from '@/utils/util'

  const props = defineProps<{ demo: Demo; isSelected: boolean; displayDetailsOnMount: boolean; levelshotPath: Nullable<string> }>()

  const emit = defineEmits<{ detailsDisplayedOnUnmount: []; showDetails: []; hideDetails: [] }>()

  const expandDetails = ref(false)
  const showServerCommands = ref(false)

  const displayDetails = computed(() => {
    return props.isSelected
  })

  const sortedServerInfo = computed(() => {
    if (props.demo == null) {
      return []
    } else {
      let sortedList = []

      let server_stuff = props.demo.server_info

      for (var key in server_stuff) {
        sortedList.push([key, server_stuff[key]])
      }

      return sortedList.sort()
    }
  })

  const sortedSystemInfo = computed(() => {
    if (props.demo == null) {
      return []
    } else {
      let sortedList = []
      let system_stuff = props.demo.system_info
      for (var key in system_stuff) {
        sortedList.push([key, system_stuff[key]])
      }

      return sortedList.sort()
    }
  })

  const serverCommandsLength = computed(() => {
    return Object.keys(props.demo.server_commands).length
  })

  const localIsSelected = ref(props.isSelected)

  watch(localIsSelected, (newVal, _oldVal) => {
    if (!newVal) {
      expandDetails.value = false
    }
  })

  function getGametype(d: Demo) {
    if (d.gamename == '') {
      return 'unknown'
    }
    if (d.gamename == 'defrag') {
      switch (props.demo.server_info['defrag_gametype']) {
        case '1':
          return 'df'
        case '2':
          return 't'
        case '3':
          return 'fc'
        case '4':
          return 'reserved'
        case '5':
          return 'mdf'
        case '6':
          return 'mt'
        case '7':
          return 'mfc'
        default:
          return 'df'
      }
    } else {
      switch (d.g_gametype) {
        case '0':
          return 'ffa'
        case '1':
          return '1v1'
        case '2':
          return 'sffa'
        case '3':
          return 'tdm'
        case '4':
          return 'ctf'
        case '5':
          return '1-flag'
        case '6':
          return 'overload'
        case '7':
          return 'harvester'
        default:
          return '???'
      }
    }
  }

  function centerHeight() {
    if (serverCommandsLength.value > 15) {
      return 'height: 60%;'
    } else {
      return 'height: max-content;'
    }
  }

  function serverCmdsHeight() {
    if (serverCommandsLength.value > 15) {
      return 'height: 101%;'
    }
  }

  onMounted(async () => {
    if (props.isSelected && props.displayDetailsOnMount) {
      expandDetails.value = true
      emit('detailsDisplayedOnUnmount')
    }
  })

  onUnmounted(async () => {
    if (displayDetails.value) {
      emit('detailsDisplayedOnUnmount')
    }
  })
</script>

<template>
  <div style="overflow: hidden" id="details">
    <div class="demo-row">
      <span style="width: 1%"></span>
      <span style="width: 9%; text-align: left; white-space: nowrap; overflow: hidden">{{ demo.gamename }}</span>
      <span style="width: 8%; text-align: left; white-space: nowrap; overflow: hidden">{{ getGametype(demo) }}</span>
      <span style="width: 12%; text-align: left; white-space: nowrap; overflow: hidden" v-html="demo.player_pov.namecolored"></span>
      <span style="width: 44%; text-align: left; white-space: nowrap; overflow: hidden">{{ demo.file_name }}</span>
      <span style="width: 1%"></span>
      <span style="width: 14%; text-align: left; white-space: nowrap; overflow: hidden">{{ demo.mapname }}</span>
      <span style="width: 1%"></span>
      <span style="width: 12%; text-align: left; white-space: nowrap">{{ demo.duration }}</span>
      <span style="width: 2%">
        <div v-if="!displayDetails" class="plus" id="expandDetails" @click="emit('showDetails')">+</div>
        <div v-if="displayDetails" class="minus" id="expandDetails" @click="emit('hideDetails')">-</div>
      </span>
    </div>

    <div v-if="displayDetails" class="demo-details">
      <img v-if="levelshotPath" class="levelshot" :src="levelshotPath" />
      <img v-else class="levelshot" src="../assets/icons/q3-white.svg" />

      <div style="width: 46%; text-align: left; white-space: nowrap; padding: 8px; overflow: hidden">
        <div>Path: {{ demo.path.slice(0, demo.path.indexOf(demo.file_name)) }}</div>
        <div>Version: {{ demo.version }}</div>
        <div>Hostname: <span v-html="demo.sv_hostname_color"></span></div>
        <div v-if="demo.issue != null">Demo issue: {{ demo.issue }}</div>
        Players:
        <div style="margin: -17px 0 0 60px; height: 126px; overflow-y: auto">
          <div v-for="(player, _index) in demo.players">
            <div style="display: inline-block; width: 65%" v-html="player.namecolored" />
          </div>
        </div>
        <br /><br />
      </div>
      <div style="width: 25%; text-align: left; white-space: nowrap; padding: 8px">
        <div style="height: 180px; overflow: hidden auto">
          Server Info:
          <div v-for="(setting, _index) in sortedServerInfo">
            <span>{{ setting[0] }}: &nbsp;</span>
            <span>{{ setting[1] }}</span>
          </div>
          System Info:
          <div v-for="(setting, _index) in sortedSystemInfo">
            <span>{{ setting[0] }}: &nbsp;</span>
            <span>{{ setting[1] }}</span>
          </div>
        </div>
      </div>

      <button class="more-button" id="moreButton" @click.prevent="showServerCommands = true">...</button>
    </div>

    <Teleport to="#popup">
      <div
        v-if="showServerCommands"
        class="backdrop"
        @mousedown.self="showServerCommands = false"
        @keydown.esc="showServerCommands = false"
      >
        <div class="center" :style="centerHeight()" @keydown.esc="showServerCommands = false">
          <h3 style="text-align: center; text-wrap: nowrap; margin: -64px 0 40px 0">{{ demo.file_name + '.dm_' + demo.protocol }}</h3>
          <div class="server-cmds" :style="serverCmdsHeight()">
            <div v-for="(serv_cmd, _index) in demo.server_commands">
              <span v-html="serv_cmd"></span>
            </div>
            <div v-if="serverCommandsLength == 0">no data</div>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<style scoped>
  .demo-row {
    color: white;
    display: flex;
    flex-direction: row;
    height: min-content;
    overflow: hidden;
  }

  .demo-details {
    height: 188px;
    display: flex;
    flex-direction: row;
    background-color: var(--secondary-bg);
    border-radius: 0.2rem;
    line-height: 18px;
    font-size: 90%;
    margin-top: 4px;
  }

  .more-button {
    background-color: rgba(0, 0, 0, 0);
    color: white;
    border: 1px solid var(--main-bg);
    border-radius: 0.2rem;
    cursor: pointer;
    font-size: 150%;
    padding: 2px 10px 2px 10px;
    font-weight: 400;
  }

  .more-button:hover {
    background-color: var(--main-bg);
  }

  .details-enter-from,
  .details-leave-active {
    opacity: 0;
    transform: scale(1);
  }
  .details-leave-active {
    position: absolute;
  }

  .details-enter-to {
    opacity: 1;
    transform: scale(1);
  }

  .details-enter-active {
    transition: all 1s ease;
  }

  .details-move {
    transition: all 1s ease;
  }

  .plus {
    padding: 0 4px 0 4px;
    font-weight: bold;
    cursor: pointer;
  }

  .minus {
    padding: 0 6px 0 7px;
    font-weight: bold;
    cursor: pointer;
  }

  .backdrop {
    top: 0;
    left: 0;
    bottom: 0;
    position: fixed;
    background: rgba(0, 0, 0, 0.7);
    width: 100%;
    z-index: 999;
    color: white;
  }

  .center {
    margin: auto;
    position: fixed;
    inset: 0px;
    width: 60%;
    text-align: left;
    color: white;
    background-color: var(--secondary-bg);
    border: 1px solid var(--main-bg);
    border-radius: 0.2rem;
    padding: 32px;
    text-wrap: wrap;
  }

  .server-cmds {
    overflow: hidden auto;
  }

  .levelshot {
    width: 250px;
    height: 188px;
    object-fit: contain;
    overflow: hidden;
  }
</style>
