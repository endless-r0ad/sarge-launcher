<script setup lang="ts">
  import { useConfig } from '@/composables/config'

  const { config } = useConfig()

</script>

<template>
  <div class="item" style="padding-bottom: 10px;">
    <input type="checkbox" v-model="config.manage_q3_instance" />
    <label class="ml-1">Manage Q3 Instance</label>
  </div>
  <div class="item" style="padding-top: 10px; border-top: 1px solid var(--main-bg);">
    <input type="checkbox" v-model="config.refresh_by_mod" />
    <label class="ml-1">Refresh by Client Game</label>
  </div>
  <div class="item">
    <input type="checkbox" v-model="config.show_unreachable" />
    <label class="ml-1">Show Unreachable Servers</label>
  </div>
  <div class="item">
    <input type="checkbox" v-model="config.show_trashed_servers" />
    <label class="ml-1">Show Trashed Servers</label>
  </div>

  <div class="conf-plus">
    +
    <label class="ml-1">Browser Threads - {{ config.server_browser_threads == 0 ? 1 : config.server_browser_threads }}</label>
  </div>
  <div class="item">
    <input type="range" min="0" max="120" step="5" value="60" class="slider" v-model.number="config.server_browser_threads" />
  </div>

  <div class="conf-plus">
    +
    <label class="ml-1">Server Timeout - {{ config.server_timeout }}ms</label>
  </div>
  <div class="item" style="padding-bottom: 10px;">
    <input type="range" min="200" max="1000" step="100" value="400" class="slider" v-model.number="config.server_timeout" />
  </div>
  <div class="item" style="padding-top: 10px; border-top: 1px solid var(--main-bg);">
    <input type="checkbox" v-model="config.autoclose_demo" />
    <label class="ml-1">Autoclose demo</label>
  </div>
  <div class="item"><input type="checkbox" v-model="config.loop_demo" />
    <label class="ml-1">Loop demo</label>
  </div>
</template>

<style scoped>
  .item {
    padding-bottom: 8px;
    text-align: left;
  }

  .conf-plus {
    margin: 0px 4px 0px 2px;
    text-align: left;
  }
</style>
