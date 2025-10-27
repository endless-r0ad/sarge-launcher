<script setup lang="ts">
  import Loading from '@/components/Loading.vue'
  import { defineProps, defineEmits, computed, onMounted, onUnmounted } from 'vue'
  import { type Quake3Server } from '@/models/server'
  import { getServerProtocol } from '@/utils/util'
  
  const props = defineProps<{
    server: Quake3Server
    isSelected: boolean
    refreshing: boolean
    altKeyHeld: boolean
    displayDetails: boolean
    displayDetailsOnMount?: boolean
    levelshotPath: string | null
  }>()

  const emit = defineEmits<{ detailsDisplayedOnUnmount: []; showDetails: []; hideDetails: [] }>()

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

  onMounted(async () => {
    if (props.isSelected && props.displayDetailsOnMount) {
      emit('detailsDisplayedOnUnmount')
    }
  })

  onUnmounted(() => {
    if (props.displayDetails && props.isSelected) {
      emit('detailsDisplayedOnUnmount')
    }
  })
</script>

<template>
  <div style="overflow: hidden">
    <div class="row-data">
      <span style="width: 2%">
        <div v-if="server.list == 'pinned' && !server.custom" class="pin-solid" id="addToListButton" />
        <div v-if="server.list == 'pinned' && server.custom" class="custom">c</div>
        <div v-if="server.list == 'main' && !altKeyHeld" class="pin" id="addToListButton" />
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
        <div v-if="!isSelected || (isSelected && !displayDetails)" class="plus" id="expandDetails" @click="emit('showDetails')">+</div>
        <div v-if="displayDetails && isSelected" class="minus" id="expandDetails" @click="emit('hideDetails')">-</div>
      </span>
    </div>

    <div v-if="displayDetails && isSelected" class="row-details">
      <img v-if="levelshotPath" class="levelshot" :src="levelshotPath" />
      <img v-else class="levelshot" src="../assets/icons/q3-white.svg" />

      <div style="width: 45%; text-align: left; white-space: nowrap; padding: 8px; overflow: hidden">
        <div>Version: {{ server.version }}</div>
        <div>Master: {{ server.master?.name }}</div>
        <div>Protocol: {{ getServerProtocol(server) }}</div>
        Players:
        <div style="margin: -17px 150px 0 60px; height: 126px; overflow-y: auto">
          <div v-for="(player, _index) in server.players">
            <div v-if="player.ping > 0" style="display: inline-block; width: 65%" v-html="player.namecolored" />
            <div v-if="player.ping > 0" style="display: inline-block">{{ player.ping }}</div>
          </div>
          <div v-for="(player, _index) in server.players">
            <div v-if="player.ping == 0" style="display: inline-block; width: 65%" v-html="player.namecolored" />
            <div v-if="player.ping == 0" style="display: inline-block">BOT</div>
          </div>
        </div>
        <br /><br />
      </div>
      <div style="width: 30%; text-align: left; white-space: nowrap; padding: 8px">
        <div style="height: 180px; overflow: hidden auto">
          <span style="font-weight: 600;">Server Info</span>
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

  .key {
    background: url('../assets/icons/key.svg') center center no-repeat;
    background-size: 18px;
  }

  .pin {
    background: url('../assets/icons/pin.svg') center center no-repeat;
    height: 20px;
    width: 20px;
    cursor: pointer;
  }

  .pin-solid,
  .pin:hover {
    background: url('../assets/icons/pin-filled.svg') center center no-repeat;
    height: 20px;
    width: 20px;
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
