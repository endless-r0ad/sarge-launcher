<script setup lang="ts">
  import DropDown from '@/components/DropDown.vue'
  import { invoke } from '@tauri-apps/api/core'
  import { defineProps, defineEmits } from 'vue'
  import { ensureError } from '@/utils/util'
  import { type Q3Executable } from '@/models/client'
  import { useConfig } from '@/composables/config'

  defineProps<{ currentView: string; }>()
  const { config, activeClient, addQ3Client } = useConfig();

  const emit = defineEmits<{
    spawnQuake: [string[]]
    errorAlert: [string]
    infoAlert: [string]
  }>()

  async function pickClient() {
    try {
      let new_client: Q3Executable = await invoke('pick_client')

      if (new_client != null) {
        if (config.value.q3_clients.some((c) => c.exe_path === new_client.exe_path)) {
          emit('infoAlert', 'client already added')
          return
        }
        addQ3Client(new_client)
      }
    } catch (err) {
      emit('errorAlert', ensureError(err).message)
    }
  }

</script>

<template>
  <div class="navbar no-select">
    <div>
      <h3 class="page-title" v-html="currentView"></h3>
    </div>

    <div class="nav-right">
      <div v-if="activeClient" class="launch-client-button" @click="emit('spawnQuake', [])" />
      <div class="add-client-button" @click="pickClient"></div>

      <DropDown />
    </div>
  </div>
</template>

<style scoped>
  .navbar {
    background-color: var(--secondary-bg);
    height: 38px;
    width: calc(100% - 146px);
    border: solid 1px #222831;
    border-radius: 0.2rem;
    position: absolute;
    top: 3vh;
    left: 112px;
    float: left;
    text-align: center;
    display: flex;
    user-select: none;
    z-index: 998;
    line-height: 38px;
  }

  .navbar div:last-child {
    margin-left: auto;
  }

  .page-title {
    color: #fff;
    padding-left: 12px;
    align-items: center;
    margin: 0 auto;
  }

  .nav-right {
    display: flex;
    justify-content: right;
    padding: 2px;
    line-height: 38px;
  }

  .launch-client-button {
    padding: 0px 16px;
    margin: 2px 4px 4px 0px;
    background: url('../assets/icons/play.svg') center center no-repeat;
  }

  .launch-client-button:hover {
    background-color: var(--main-bg);
    border-radius: 0.2rem;
    cursor: pointer;
  }

  .add-client-button {
    padding: 0px 16px;
    margin: 2px 4px 4px 0px;
    background: url('../assets/icons/plus.svg') center center no-repeat;
    cursor: pointer;
  }

  .add-client-button:hover {
    background-color: var(--main-bg);
    border-radius: 0.2rem;
  }
</style>
