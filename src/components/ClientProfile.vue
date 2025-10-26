<script setup lang="ts">
  import type { Q3Executable, Q3Config } from '@/models/client';
  import { defineProps, defineEmits, onMounted, onBeforeUnmount, ref, computed, watch } from 'vue'
  import { useClient } from '@/composables/client';
  import { invoke } from '@tauri-apps/api/core'

  const props = defineProps<{ profiledClient: Q3Executable }>()

  const emit = defineEmits<{ cancelModal: []; executeModal: []; mutateConfig: []; deleteClient: [] }>()

  const { 
    getClientConfigs,
    getClientGameProtocol,
    getClientDefaultGamename,
    updateClientGame,
    activeClient,
    toggleQ3Client
  } = useClient()

  const localClient = ref(props.profiledClient)

  watch(() => localClient.value.gamename, (_newVal, _oldVal) => {
    configs.value = []
  })

  const mountedClientGame = ref(props.profiledClient.gamename)

  const defaultClientGame = ref('')
  const additionalLaunchOptions = ref('')

  function handleKeyPress(event: KeyboardEvent) {
    if (event.code == 'Escape') {
      emit('cancelModal')
    }
  }

  async function reveal(p: string) {
    await invoke('reveal_item_in_dir', {path: p})
  }

  const configs = ref<Q3Config[]>([])

  async function getConfigs() {
    configs.value = await getClientConfigs(localClient.value)
  }

  const sortedConfigs = computed(() => {
    return [...configs.value].sort((a, b) => {
      if (a.name.toLowerCase() < b.name.toLowerCase()) {
        return -1
      }
      if (a.name.toLowerCase() > b.name.toLowerCase()) {
        return 1
      }
      return 0
    })
  })

  onMounted(() => {
    document.addEventListener('keydown', handleKeyPress)
    defaultClientGame.value = getClientDefaultGamename(localClient.value)
  })

  onBeforeUnmount( async() => {
    document.removeEventListener('keydown', handleKeyPress)
    if (mountedClientGame.value != localClient.value.gamename) {
      await updateClientGame(localClient.value)
      if (localClient.value === activeClient.value) {
        toggleQ3Client(localClient.value) // retoggle to update search paths
      }
    }
  })
</script>

<template>
  <div style="width: 540px; height: 340px">
    <h2 style="margin: 8px 0px">{{ localClient.name }}</h2>
    <div class="profile-item" style="float: right; position: absolute; right: 20px; top: 32px">
      <button class="refresh-button" style="font-size: 90%; border-color: orange" @click="emit('deleteClient')">Remove Client</button>
    </div>
    <div class="profile-item">
      <button class="refresh-button" style="font-size: 90%" @click="reveal(localClient.exe_path)">{{ localClient.parent_path }}</button>
    </div>
    <div class="profile-item">
      <label class="ml-1">Protocol {{ getClientGameProtocol(localClient) }}</label>
    </div>
    <div class="profile-item">
      <label class="ml-1">fs_game override</label>
      <input
        type="text"
        v-model="localClient.gamename"
        class="search"
        style="width: 80px; margin-left: 16px"
        :style="localClient.gamename != defaultClientGame ? 'outline: 1px solid orange;' : ''"
      />
      <button
        v-if="localClient.gamename != defaultClientGame"
        class="refresh-button"
        style="margin-left: 16px; font-size: 100%"
        @click="localClient.gamename = defaultClientGame"
      >
        reset
      </button>
    </div>
    <div class="profile-item">
      <label class="ml-1">Additional launch options</label>
      <input type="text" v-model="additionalLaunchOptions" class="search" style="width: 180px; margin-left: 16px" />
    </div>
    <div class="profile-item">
      <button class="refresh-button" @click="getConfigs()" style="font-size: 90%">Browse Configs</button>
    </div>
    
    <div style="height: 140px; overflow-y: scroll">
      <div v-for="(c, i) in sortedConfigs" 
          @click="reveal(c.path)" 
          class="row config" 
          :style="i % 2 ? 'background-color: rgba(23, 32, 45, 0.3);' : ''"
          :key="c.path + c.name">
        {{ c.name }}
      </div>
    </div>
  </div>
</template>

<style scoped>
  .config {
    font-size: 80%;
    padding-left: 8px;
  }

  .config:hover {
    font-size: 80%;
    background-image: linear-gradient(to right, rgba(0, 143, 168, 0.514) 1%, rgba(255, 255, 255, 0.02));
    cursor: pointer;
  }

  .profile-item {
    margin-bottom: 4px;
    line-height: 26px;
  }

</style>
