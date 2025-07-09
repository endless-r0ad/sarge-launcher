<script setup lang="ts">
  import Loading from '@/components/Loading.vue'
  import { watch, defineProps, defineEmits, ref, computed, onMounted, onUnmounted } from 'vue'
  import { type Quake3Server } from '@/models/server'
  import { type Nullable } from '@/utils/util'

  const props = defineProps<{
    server: Quake3Server
    isSelected: boolean
    refreshing: boolean
    altKeyHeld: boolean
    displayDetailsOnMount?: boolean
    levelshotPath: Nullable<string>
  }>()

  const emit = defineEmits<{ detailsDisplayedOnUnmount: []; showDetails: []; hideDetails: [] }>()

  const expandDetails = ref(false)

  const displayDetails = computed(() => {
    return props.isSelected
  })

  const requiresPassword = computed(() => {
    if ('g_needpass' in props.server.othersettings && props.server.othersettings['g_needpass'] == '1') {
      return true
    }
    return false
  })

  const sortedOtherSettings = computed(() => {
    if (props.server == null) {
      return []
    }
    let sortedList = []

    for (var key in props.server.othersettings) {
      sortedList.push([key, props.server.othersettings[key]])
    }

    return sortedList.sort()
  })

  const localIsSelected = ref(props.isSelected)

  watch(localIsSelected, (newVal, _oldVal) => {
    if (!newVal) {
      expandDetails.value = false
    }
  })

  function getServerProtocol(serv: Quake3Server) {
    if (serv.protocol) {
      return serv.protocol
    }
    if (serv.othersettings.hasOwnProperty('protocol')) {
      return serv.othersettings['protocol']
    }
  }

  onMounted(async () => {
    if (props.isSelected && props.displayDetailsOnMount) {
      expandDetails.value = true
      emit('detailsDisplayedOnUnmount')
    }
  })

  onUnmounted(() => {
    if (displayDetails.value) {
      emit('detailsDisplayedOnUnmount')
    }
  })
</script>

<template>
  <div style="overflow: hidden">
    <div class="server-row">
      <span style="width: 2%">
        <div v-if="server.list == 'pinned' && !server.custom" class="fave-solid" id="addToListButton" />
        <div v-if="server.list == 'pinned' && server.custom" class="custom">c</div>
        <div v-if="server.list == 'main' && !altKeyHeld" class="fave" id="addToListButton" />
        <div v-if="server.list == 'main' && altKeyHeld" class="trash-button" id="addToListButton" />
        <div v-if="server.list == 'trash'" class="minus" id="addToListButton">-</div>
      </span>
      <span style="width: 1%"></span>
      <span style="width: 11%" class="data">{{ server.game }}</span>
      <span style="width: 3%; text-align: center" :class="requiresPassword ? 'key' : ''" />
      <span v-html="server.hostcolored" style="width: 36%" class="data"></span>
      <span style="width: 1%"></span>
      <span style="width: 16%" class="data">{{ server.map }}</span>
      <span style="width: 10%" class="data">{{ server.playersconnected }}/{{ server.maxclients }}</span>
      <span style="width: 2%" class="data">
        <Loading v-if="refreshing && isSelected" :position="'relative'" :size="15" />
      </span>
      <span style="width: 7%" class="data">{{ server.ping }}</span>
      <span style="width: 16%" class="data">{{ server.ip }}:{{ server.port }}</span>
      <span style="width: 2%">
        <div v-if="!displayDetails" class="plus" id="expandDetails" @click="emit('showDetails')">+</div>
        <div v-if="displayDetails" class="minus" id="expandDetails" @click="emit('hideDetails')">-</div>
      </span>
    </div>

    <div v-if="displayDetails" class="server-details">
      <img v-if="levelshotPath" class="levelshot" :src="levelshotPath" />
      <img v-else class="levelshot" src="../assets/icons/q3-white.svg" />

      <div style="width: 45%; text-align: left; white-space: nowrap; padding: 8px; overflow: hidden">
        <div>Version: {{ server.version }}</div>
        <div>Master: {{ server.master?.name }}</div>
        <div>Protocol: {{ getServerProtocol(server) }}</div>
        Players:
        <div style="margin: -17px 150px 0 60px; height: 126px; overflow-y: auto">
          <div v-for="(player, _index) in server.players">
            <div style="display: inline-block; width: 65%" v-html="player.namecolored" />
            <div v-if="player.ping > 0" style="display: inline-block">{{ player.ping }}</div>
            <div v-else style="display: inline-block">BOT</div>
          </div>
        </div>
        <br /><br />
      </div>
      <div style="width: 30%; text-align: left; white-space: nowrap; padding: 8px">
        <div style="height: 180px; overflow: hidden auto">
          Server Info:
          <div v-for="(setting, _index) in sortedOtherSettings">
            <span>{{ setting[0] }}: &nbsp;</span>
            <span>{{ setting[1] }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
  .trash-button {
    background: url('../assets/icons/trash.svg') center center no-repeat;
    background-size: 14px 14px;
    height: 20px;
    cursor: pointer;
  }

  .trash-button:hover {
    background: url('../assets/icons/trash-filled.svg') center center no-repeat;
    background-size: 14px 14px;
    height: 20px;
    cursor: pointer;
  }

  .server-row {
    color: white;
    display: flex;
    flex-direction: row;
    height: min-content;
    overflow: hidden;
  }

  .server-details {
    height: 188px;
    display: flex;
    flex-direction: row;
    background-color: var(--secondary-bg);
    border-radius: 0.2rem;
    line-height: 18px;
    font-size: 90%;
    margin-top: 4px;
  }

  .server-details-right {
    float: right;
    width: 30%;
  }

  .server-details-left {
    display: block;
    margin: 0 71% 0 0;
  }

  .key {
    background: url('../assets/icons/key.svg') center center no-repeat;
    background-size: 18px;
  }

  .trash-row {
    opacity: 0.5;
  }

  .fave {
    background: url('../assets/icons/fave.svg') center center no-repeat;
    height: 20px;
    cursor: pointer;
  }

  .fave:hover {
    background: url('../assets/icons/fave-filled.svg') center center no-repeat;
    height: 20px;
    cursor: pointer;
  }

  .fave-solid {
    background: url('../assets/icons/fave-filled.svg') center center no-repeat;
    height: 20px;
    cursor: pointer;
  }

  .page-title {
    text-align: left;
    padding-bottom: 5px;
    border-bottom: 4px solid #fa8225;
    flex: 1 1 auto;
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

  .server-left {
    display: flex;
    margin: 0 98% 0 0;
  }

  .server-right {
    float: right;
    width: 98%;
    overflow: hidden;
  }

  .data {
    text-align: left;
    white-space: nowrap;
    overflow: hidden;
  }

  .minus {
    padding: 0 6px 0 7px;
    font-weight: bold;
    cursor: pointer;
  }

  .custom {
    padding: 0 6px 0 7px;
    font-weight: 500;
  }

  .plus {
    padding: 0 4px 0 4px;
    font-weight: bold;
    cursor: pointer;
  }

  .levelshot {
    width: 250px;
    height: 188px;
    object-fit: contain;
    overflow: hidden;
    aspect-ratio: 4/3;
  }
</style>
