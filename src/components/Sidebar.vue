<script setup lang="ts">
  import { defineProps, defineEmits } from 'vue'

  const props = defineProps<{ showSettings: boolean }>()
  const emit = defineEmits<{ toggleSettings: []; exitApp: [] }>()
</script>

<template>
  <div class="sidebar">
    <div class="logo">
      <router-link to="/home" style="background: transparent">
        <img v-if="$route.path == '/home'" src="../assets/icons/q3-accent.svg" width="50px" />
        <img v-else src="../assets/icons/q3-white.svg" width="50px" />
      </router-link>
    </div>

    <div>
      <router-link to="/singleplayer" class="link">
        <span v-if="$route.path == '/singleplayer'" class="blue-dot"></span>
        <img src="../assets/icons/single-player.svg" width="32px" height="32px" />
      </router-link>

      <div style="margin: 8px"></div>

      <router-link to="/server" class="link">
        <span v-if="$route.path == '/server'" class="blue-dot"></span>
        <img src="../assets/icons/globe.svg" width="31px" height="31px" />
      </router-link>

      <div style="margin: 8px"></div>

      <router-link to="/demo" class="link">
        <span v-if="$route.path == '/demo'" class="blue-dot"></span>
        <img src="../assets/icons/replay.svg" width="32px" height="32px" />
      </router-link>

      <div style="margin: 8px"></div>

      <div class="link" :class="{ settings: props.showSettings }" @click="emit('toggleSettings')">
        <span v-if="props.showSettings"></span>
        <img src="../assets/icons/settings.svg" width="32px" height="32px" />
      </div>
    </div>

    <div class="sidebar-line"></div>

    <div class="exit">
      <img src="../assets/icons/x.svg" @click="emit('exitApp')" width="20px" />
    </div>
  </div>
</template>

<style scoped>
  .sidebar {
    color: white;
    position: fixed;
    z-index: 999;
    text-align: center;
    width: 72px;
    top: 50%;
    left: 0;
    transform: translateY(-50%);
    display: flex;
    flex-direction: column;
    height: 96vh;
    opacity: 1;
    border-radius: 10px;
    margin-left: 6px;
    justify-content: center;
    align-items: center;
    user-select: none;
  }

  .logo {
    padding: 12px;
    position: absolute;
    top: 0;
    text-align: center;
  }

  .exit {
    display: block;
    text-align: center;
    border-radius: 0.8rem;
    position: absolute;
    bottom: 10px;
    text-align: center;
    cursor: pointer;
  }

  .sidebar-line {
    color: white;
    background-color: var(--secondary-bg);
    float: left;
    position: fixed;
    z-index: 1;
    text-align: center;
    width: 1px;
    top: 50%;
    transform: translateY(-50%);
    margin-left: 82px;
    display: flex;
    flex-direction: column;
    height: 94vh;
    opacity: 1;
  }

  .sidebar-icon {
    width: 128px;
    display: block;
  }

  .settings-btn {
    position: absolute;
    bottom: 10px;
    width: 24px !important;
    margin-left: 15px;
    text-align: center;
    justify-content: center;
    align-items: center;
    cursor: pointer;
  }

  .settings-btn:hover {
    border: 1px solid var(--sidebar-item-hover);
  }

  .sidebar-divider {
    border-bottom: 0.01em solid #52698a;
    margin: 4px 12% 4px 12%;
  }

  .header {
    background-color: rgba(46, 59, 79, 1);
    border-right: 0.01em solid #17202d;
    border-bottom: 0.01em solid #17202d;
    height: 64px;
  }

  .sidebar-title {
    text-align: center;
  }

  .profile {
    padding: 0.75em;
    text-align: left;
    width: 10%;
    color: rgba(46, 59, 79, 1);
    cursor: pointer;
    margin: 80px, auto;
  }

  .rotate-180 {
    transform: rotateY(180deg);
    transform: 3s linear;
  }

  .fade-enter-active {
    transition: opacity 10s;
  }

  .fade-enter-on {
    opacity: 0;
  }

  .centered-element {
    margin: auto;
    position: absolute;
    top: 50%;
    transform: translateY(-50%);
  }

  .link {
    display: block;
    padding: 8px 8px 2px 8px;
    text-align: center;
    border-radius: 0.8rem;
    cursor: pointer;
  }

  .link:hover {
    background-color: var(--secondary-bg);
  }

  .settings {
    display: block;
    padding: 8px 8px 2px 8px;
    text-align: center;
    border-radius: 0.8rem;
    background-color: var(--secondary-bg);
  }

  .blue-dot {
    position: fixed;
    left: -6px;
    margin-top: 12px;
    border: 4px solid #00ffff;
    border-radius: 50%;
    cursor: default;
  }
</style>
