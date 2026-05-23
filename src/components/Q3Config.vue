<script setup lang="ts">
  import { watch, ref, computed } from 'vue'
  import { useClient } from '@/composables/client'

  const { activeClient, activeClientQ3Config } = useClient()

  const sortedQ3Config = computed(() => {
    let sortedConf = []
    let seta = activeClientQ3Config.value['seta']

    if (seta) {
      for (var key in seta) {
        if (key == 'vhtml_name') { continue }
        sortedConf.push([key, seta[key] ?? ''])
      }
    }
    return sortedConf.sort()
  })
  
  const searchQuery = ref('')
  const entries = ref<string[][]>(sortedQ3Config.value)

  watch(searchQuery, async (newSearch, _oldSearch) => {

    let query = newSearch.toLowerCase()

    let filtered: string[][] = []

    for (let i = 0; i < sortedQ3Config.value.length; i++) {
      if (sortedQ3Config.value[i]![0]!.toLowerCase().includes(query)) {
        filtered.push([sortedQ3Config.value[i]![0]!, sortedQ3Config.value[i]![1]!])
      }
    }

    entries.value = filtered
  })

</script>

<template>
  <div style="height: 500px; width: 400px;">
    <div class="table-header-base no-select" style="height: 50px;">
      <div class="table-header-right">
        <input class="search" type="text" placeholder="search" v-model="searchQuery" />
      </div>
      <div class="table-header-left">
        {{activeClient?.name}} q3config.cfg
      </div>
    </div>
    <div style="overflow-y: scroll; height: calc(100% - 44px);">
      <div v-for="cvar in entries" :key="cvar[0]">
        <span>{{ cvar[0] }}</span> <span style="float: right;">{{ cvar[1] }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>

</style>
