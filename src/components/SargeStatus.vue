<script setup lang="ts">

  defineProps<{ updateAvailable: boolean, welcomeMessage: boolean, appVersion: string, latestRelease: string | null }>()

</script>

<template>
  <div style="display: flex; align-items: center; justify-content: center; width: 320px;">
    <img src="../assets/icons/sarge.svg" />
    <h2>SARGE LAUNCHER</h2>
  </div>
  <a v-if="updateAvailable" href="https://github.com/endless-r0ad/sarge-launcher/releases" target="_blank">
    <div class="update-available">
        <span style="color: #00ffff;">update available</span>
        <span>{{ appVersion }}</span>
        <span style="color: #00ffff;">-> {{ latestRelease }}</span>
    </div>
  </a>
  <div v-else style="text-align: right; margin-top: -16px;">
    <a href="https://github.com/endless-r0ad/sarge-launcher" target="_blank">
      {{ appVersion }}
    </a>
  </div>
  <div v-if="welcomeMessage" style="width: 320px;">
    <p>
      Sarge Launcher is a custom frontend for Quake 3 Arena and Q3A mods that provides some useful features for both n00bs and
      veterans.
    </p>
    <p>
      Get started by linking Sarge Launcher to your Q3 client(s) on the Client Dash
    </p>
    <p>
      If you do not have Quake 3 yet you should purchase the game, then update your client to ioquake3 or quake3e.
    </p>
    <p>
      Sarge Launcher is not responsible for installing any mods or downloading any content for Q3A -
      YOU are solely responsible for that!
    </p>
    <div style="display: flex; align-items: center; justify-content: center;">
      <a style="font-size: 90%" href="https://github.com/endless-r0ad/sarge-launcher" target="_blank">
        <button class="refresh-button">Github</button>
      </a>
      <a style="font-size: 90%" href="https://github.com/endless-r0ad/sarge-launcher/issues" target="_blank">
        <button class="refresh-button" >Issues</button>
      </a>
      <a style="font-size: 90%" href="https://github.com/endless-r0ad/sarge-launcher/releases" target="_blank">
        <button class="refresh-button" >Releases</button>
      </a>
    </div>
  </div>
  <div v-else style="margin: 12px;"></div>

</template>

<style scoped>
  a {
    text-decoration: none;
    color: white;
    z-index: 998;
    font-size: 82%;
    font-weight: bold;
  }

  .update-available {
    display: flex; 
    align-items: center; 
    justify-content: right; 
    margin-top: -16px; 
    gap: 4px;
    background-color: red;
  }

  .update-available:hover {
    background-color: rgba(0, 0, 0, 0);
    cursor: pointer;
  }

</style>