<script setup lang="ts">
  import type { MasterServer } from '@/models/master'
  import { defineEmits, onBeforeUnmount, ref, toRaw, type Ref, onUpdated, computed } from 'vue'
  import { useAppData } from '@/composables/appdata'

  const props = defineProps<{ q3MasterProtocol: number }>()
  const emit = defineEmits<{ fullRefresh: []; toggleProtocol: [] }>()

  const { appdata, updateMasterSettings } = useAppData()

  const localMasterSettings = ref(structuredClone(toRaw(appdata.value.masters))) as Ref<MasterServer[]>
  const fullRefreshNeeded = ref(false)
  const protocolChanged = ref(false)
  const mountedMasterSettings = structuredClone(toRaw(appdata.value.masters))
  const mountedProtocol = props.q3MasterProtocol

  const q3MasterIsActive = computed(() => {
    return localMasterSettings.value.some((m) => m.game === 'Quake 3' && m.active)
  })

  onUpdated(async () => {
    let appdataNeedsUpdate = false
    protocolChanged.value = mountedProtocol != props.q3MasterProtocol
    fullRefreshNeeded.value = false

    localMasterSettings.value.forEach((s) => {
      const match = appdata.value.masters.find((m) => m.address === s.address && m.game === s.game)
      if (match && match.active != s.active) {
        appdataNeedsUpdate = true
      }
    })

    mountedMasterSettings.forEach((s) => {
      const match = localMasterSettings.value.find((m) => m.address === s.address && m.game === s.game)
      if (match && match.active != s.active) {
        fullRefreshNeeded.value = true
      }
    })

    if (appdataNeedsUpdate) {
      await updateMasterSettings(localMasterSettings.value)
    }
  })

  onBeforeUnmount(async () => {
    if (fullRefreshNeeded.value || (protocolChanged.value && q3MasterIsActive.value)) {
      emit('fullRefresh')
    }
  })
</script>

<template>
  <div v-for="master in localMasterSettings" style="height: 32px">
    <span><input type="checkbox" v-model="master.active" /></span>
    <text :style="master.unreachable ? 'color: #aaa; text-decoration: line-through;' : ''" class="ml-1"
      >{{ master.game }}: {{ master.name }}
    </text>
  </div>
  <div class="protocol">
    <text>+</text>
    <text class="ml-1">Quake 3 Master Protocol</text>
    <text class="ml-1"><input type="radio" :value="Number(68)" @change="emit('toggleProtocol')" :checked="q3MasterProtocol == 68" />68</text>
    <text class="ml-1"><input type="radio" :value="Number(43)" @change="emit('toggleProtocol')" :checked="q3MasterProtocol == 43" />43</text>
  </div>
</template>

<style scoped>
  .protocol {
    height: 32px;
    margin: 0px 4px 0px 2px;
    text-align: left;
  }
</style>
