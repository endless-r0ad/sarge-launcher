<script setup lang="ts">
  import Popup from "@/components/Popup.vue"
  import { defineProps, defineEmits, ref, onMounted } from "vue"
  import { invoke } from "@tauri-apps/api/core"
  import { ensureError } from "@/utils/util"
  import { type Config, type AppData } from "@/models/config"
  import { type Q3Executable } from "@/models/client"

  const props = defineProps<{ config: Config; appData: AppData, showUnreachableServers: boolean, showTrashedServers: boolean }>()

  const emit = defineEmits<{
    mutateConfig: [Config]
    mutateAppData: [AppData]
    spawnQuake: [string]
    addQ3Client: [Q3Executable]
    emitConnectArgs: [string[]]
    emitComponentName: [string]
    errorAlert: [string]
    infoAlert: [string]
  }>()

  const componentName = ref("sarge-launcher")
  const showWelcomeMessage = ref<boolean>(props.config.welcome_message)

  function closeWelcomeMessage() {
    showWelcomeMessage.value = false

    if (props.config.welcome_message) {
      emit("mutateConfig", { ...props.config, welcome_message: false })
    }
  }

  const hoveredCard = ref("")

  // async function FindQ3Dirs() { await invoke('find_q3_dirs'); }

  // function spawnQuake(){ emit('spawnQuake', componentName.value) }

  function emitComponentName() {
    emit("emitComponentName", componentName.value)
  }

  async function pickClientBlocking() {
    try {
      let new_client: Q3Executable = await invoke("pick_client_blocking")

      if (new_client != null) {
        emit("addQ3Client", new_client)
      }
    } catch (err) {
      emit("errorAlert", ensureError(err).message)
    }
  }

  onMounted(() => emitComponentName())
</script>

<template>
  <Teleport to="#popup">
    <Popup v-if="showWelcomeMessage" :popupType="'center'" @cancelModal="closeWelcomeMessage">
      <div style="width: 400px;">
        <img style="position: absolute; left: 15%; top: 4%;" src="../assets/icons/sarge.svg">
        <h2 style="position: absolute; right: 15%; top: 4%;">SARGE LAUNCHER</h2>
        <p style="margin-top: 72px;">
          Sarge Launcher is a utility for Quake 3 Arena and Quake 3 Arena mods that provides some useful features for both n00bs and
          veterans. If you are new, you should purchase the game first, then update your client to ioquake3 or quake3e.
        </p>
        <p>Once you have Q3A you can link the client or mod client using +</p>
        <p>
          Sarge Launcher is not responsible for installing any mods, modifying any configs, or downloading any content for Quake 3 Arena -
          YOU are solely responsible for that!
        </p>
      </div>
    </Popup>
  </Teleport>

  <div class="client-grid" draggable="false">
    <div
      class="grid-bg welcome-bg grow"
      @mouseover="hoveredCard = 'welcome'"
      @mouseleave="hoveredCard = ''"
      @click="showWelcomeMessage = true"
      style="grid-column: 1; grid-row: 1"
    >
      <div v-if="hoveredCard == 'welcome'" class="tint">
        <span class="center card-name" draggable="false">{{ hoveredCard }}</span>
      </div>
    </div>
    <div
      class="grid-bg grow"
      @mouseover="hoveredCard = 'quake 3 arena'"
      @mouseleave="hoveredCard = ''"
      style="grid-column: 2 / 5; grid-row: 1 / 3"
    >
      <a class="link" href="https://www.youtube.com/watch?v=ZH51hb7hIN0&list=PL20723923F60F926A" target="_blank">
        <!-- <video src="../assets/hero_vid.mp4" autoplay muted class="q3-video" type="video/mp4" onloadstart="this.playbackRate = 0.25;"></video> -->
        <img v-if="config.play_gif" src="../assets/images/orbb-quake.gif" class="q3-video" />
        <img v-if="!config.play_gif" src="../assets/images/q3map2.png" class="q3-video" />
        <div v-if="hoveredCard == 'quake 3 arena'" class="center card-name tint">
          <span class="center">{{ hoveredCard }}</span>
          <div class="play-pause" @click="config.play_gif = !config.play_gif">
            <img v-if="config.play_gif" src="../assets/icons/pause.svg" />
            <img v-else src="../assets/icons/play.svg" />
          </div>
          <!-- <button class="top-right" @click="this.config.play_gif = !this.config.play_gif">play / pause</button>    -->
        </div>
      </a>
    </div>
    <div
      class="grid-bg plus-bg grow"
      @mouseover="hoveredCard = 'add client'"
      @mouseleave="hoveredCard = ''"
      style="cursor: pointer; background-color: var(--secondary-bg); grid-column: 5; grid-row: 1"
    >
      <div v-if="hoveredCard == 'add client'" class="tint" @click="pickClientBlocking">
        <span class="center card-name">{{ hoveredCard }}</span>
      </div>
    </div>
    <div
      class="grid-bg quake-bg grow"
      @mouseover="hoveredCard = 'buy'"
      @mouseleave="hoveredCard = ''"
      style="grid-column: 1; grid-row: 2 / 4"
    >
      <a class="link" href="https://store.steampowered.com/app/2200/Quake_III_Arena/" target="_blank">
        <div v-if="hoveredCard == 'buy'" class="tint spotlight">
          <div class="center card-name">buy</div>
          <!-- <div class="center-gog card-name">gog</div> -->
          <!-- <span class="center card-name">steam</span>
          <span class="center card-name">gog</span> -->
        </div>
      </a>
    </div>
    <!-- <div class="grid-bg quake-bg grow" @mouseover="hoveredCard='gog'" @mouseleave="hoveredCard=''" style="grid-column: 3; grid-row: 3 / 5;">
      <a class="link" href="https://www.gog.com/en/game/quake_iii_arena" target="_blank">
        <div v-if="hoveredCard=='gog'" class="tint">
          <span class="center card-name">{{ hoveredCard }}</span>
        </div> 
      </a>
    </div> -->
    <div
      class="grid-bg upgrade-bg grow"
      @mouseover="hoveredCard = 'quake3e'"
      @mouseleave="hoveredCard = ''"
      style="grid-column: 4; grid-row: 3"
    >
      <a class="link" href="https://github.com/ec-/Quake3e" target="_blank">
        <div v-if="hoveredCard == 'quake3e'" class="tint">
          <span class="center card-name">{{ hoveredCard }}</span>
        </div>
      </a>
    </div>
    <div
      class="grid-bg map-bg grow"
      @mouseover="hoveredCard = 'quake 3 mapping'"
      @mouseleave="hoveredCard = ''"
      style="grid-column: 1 / 3; grid-row: 4"
    >
      <a class="link" href="https://trello.com/b/zJp4pE3m/id-tech-3-mapping" target="_blank">
        <div v-if="hoveredCard == 'quake 3 mapping'" class="tint spotlight">
          <span class="center card-name">{{ hoveredCard }}</span>
        </div>
      </a>
    </div>
    <div
      class="grid-bg banner-bg grow"
      @mouseover="hoveredCard = 'quake 3 mods'"
      @mouseleave="hoveredCard = ''"
      style="grid-column: 2 / 4; grid-row: 3"
    >
      <a class="link" href="https://lvlworld.com/mods" target="_blank">
        <div v-if="hoveredCard == 'quake 3 mods'" class="tint spotlight">
          <span class="center card-name">{{ hoveredCard }}</span>
        </div>
      </a>
    </div>
    <div
      class="grid-bg upgrade-bg grow"
      @mouseover="hoveredCard = 'ioquake3'"
      @mouseleave="hoveredCard = ''"
      style="grid-column: 3; grid-row: 4"
    >
      <a class="link" href="https://ioquake3.org/" target="_blank">
        <div v-if="hoveredCard == 'ioquake3'" class="tint">
          <span class="center card-name">{{ hoveredCard }}</span>
        </div>
      </a>
    </div>
    <div
      class="grid-bg urt-bg grow"
      @mouseover="hoveredCard = 'urban terror'"
      @mouseleave="hoveredCard = ''"
      style="grid-column: 5; grid-row: 3"
    >
      <a class="link" href="https://www.urbanterror.info/home/" target="_blank" rel="noopener">
        <div v-if="hoveredCard == 'urban terror'" class="tint">
          <span class="center card-name">{{ hoveredCard }}</span>
        </div>
      </a>
    </div>
    <div class="grid-bg cpma-bg grow" @mouseover="hoveredCard = 'cpma'" @mouseleave="hoveredCard = ''" style="grid-column: 5; grid-row: 2">
      <a class="link" href="https://playmorepromode.com/" target="_blank">
        <div v-if="hoveredCard == 'cpma'" class="tint">
          <span class="center card-name">{{ hoveredCard }}</span>
        </div>
      </a>
    </div>
    <div
      class="grid-bg defrag-bg grow"
      @mouseover="hoveredCard = 'defrag'"
      @mouseleave="hoveredCard = ''"
      style="grid-column: 4; grid-row: 4"
    >
      <a class="link" href="https://defrag.racing/" target="_blank">
        <div v-if="hoveredCard == 'defrag'" class="tint">
          <span class="center card-name">{{ hoveredCard }}</span>
        </div>
      </a>
    </div>
    <div
      class="grid-bg oa-bg grow"
      @mouseover="hoveredCard = 'openarena'"
      @mouseleave="hoveredCard = ''"
      style="grid-column: 5; grid-row: 4"
    >
      <a class="link" href="https://openarena.ws/news.html" target="_blank">
        <div v-if="hoveredCard == 'openarena'" class="tint">
          <span class="center card-name">{{ hoveredCard }}</span>
        </div>
      </a>
    </div>
  </div>
</template>

<style scoped>
  .client-grid {
    /* width: 100%; */
    height: 100%;
    display: grid;
    grid-template-columns: repeat(5, minmax(111px, 20%));
    grid-template-rows: repeat(4, minmax(74px, 25%));
    /* justify-items: center; */
    grid-gap: 16px;
    /* grid-auto-flow: row dense; */
    /* text-align: center;
    align-items: center; */
    padding: 4px;
    color: white;
    user-select: none;
    /* overflow: hidden; */
  }

  .grid-bg {
    /* background-color: linear-gradient(135deg, rgba(255, 255, 255, 0.3) 1%, rgba(255, 255, 255, 0.001) 25.35%); */
    background-color: var(--secondary-bg);
    background-repeat: no-repeat;
    background-size: 100%;
    background-position: center center;
    color: white;
    /* border-radius: 0.3rem; */
  }

  .plus-bg {
    background-image: url("../assets/icons/plus.svg");
    background-size: 10%;
    /* background-size: 102%; */
  }

  .banner-bg {
    background-image: url("../assets/images/banner-2.jpg");
    background-size: 102%;
  }

  .quake-bg {
    background-image: url("../assets/images/q3_box_art.jpg");
  }

  .welcome-bg {
    background-image: url("../assets/icons/sarge.svg");
    background-size: 40%;
  }

  .upgrade-bg {
    background-image: url("../assets/icons/q3-white.svg");
    background-size: 40%;
  }

  .urt-bg {
    background-image: url("../assets/images/urt.png");
    background-size: 60%;
  }

  .defrag-bg {
    background-image: url("../assets/images/idfe_logo01.svg");
    background-size: 220%;
  }

  .cpma-bg {
    background-image: url("../assets/images/cpma-2.png");
    background-size: 60%;
  }

  .map-bg {
    background-image: url("../assets/images/map-nightmare.jpg");
    background-position: center center;
    /* border-bottom: 1px solid white; */
  }

  .mod-bg {
    /* background-image: url('../assets/images/mod-code.png'); */
    background-position: center center;
    /* border-bottom: 1px solid white; */
  }

  .oa-bg {
    background-image: url("../assets/images/Openarena.svg");
    background-size: 70%;
  }

  .grow {
    transition: all 0.35s ease-in-out;
  }

  .grow:hover {
    transform: scale(1.02);
    cursor: pointer;
  }

  .tint {
    /* transition: all .2s ease-in-out;  */
    min-height: 100%;
    min-width: 100%;
    background-color: rgba(0, 0, 0, 0.6);
  }

  .spotlight {
    background-image: linear-gradient(135deg, rgba(255, 255, 255, 0.7) 1%, rgba(255, 255, 255, 0.001) 25.35%);
  }

  .play-pause {
    position: absolute;
    left: 94%;
    top: 10%;
    transform: translate(-50%, -50%);
    text-align: center;
    z-index: 999;
  }

  .center {
    position: absolute;
    left: 50%;
    top: 50%;
    transform: translate(-50%, -50%);
    text-align: center;
    min-width: 100%;
  }

  .center-steam {
    position: absolute;
    left: 50%;
    top: 40%;
    transform: translate(-50%, -50%);
    text-align: center;
    min-width: 100%;
  }

  .center-gog {
    position: absolute;
    left: 50%;
    top: 60%;
    transform: translate(-50%, -50%);
    text-align: center;
    min-width: 100%;
  }

  .card-name {
    font-size: 150%;
    font-weight: bold;
  }

  .link {
    color: white;
    z-index: 998;
  }

  .q3-video {
    display: block;
    top: 0;
    bottom: 0;
    right: 0;
    left: 0;
    width: 100%;
    height: 100%;
  }

  .q3-video:hover {
    background-color: rgba(0, 0, 0, 0.7);
    /* min-height: 100%; */
  }
</style>
