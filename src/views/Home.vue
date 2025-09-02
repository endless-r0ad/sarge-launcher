<script setup lang="ts">
  import Modal from '@/components/Modal.vue'
  import { defineEmits, ref, onMounted, onActivated } from 'vue'
  import { ensureError } from '@/utils/util'
  import { useConfig } from '@/composables/config'
  import { useClient } from '@/composables/client'

  const emit = defineEmits<{spawnQuake: [string[]], emitComponentName: [string], errorAlert: [string], infoAlert: [string]}>()

  const componentName = ref('Sarge Launcher')

  const { config } = useConfig()
  const { pickClient } = useClient()

  const hoveredCard = ref('')

  async function pickQ3Client() {
    try {
      let isNewClient = await pickClient()
      if (!isNewClient) {
        emit('infoAlert', 'client already added')
      }
    } catch (err) {
      emit('errorAlert', ensureError(err).message)
    }
  }

  onMounted(() => emit('emitComponentName', componentName.value))
  onActivated(() => emit('emitComponentName', componentName.value))
</script>

<template>
  <Teleport to="#modal">
    <Modal v-if="config.welcome_message" :popupType="'center'" @cancelModal="config.welcome_message = false">
      <div style="width: 400px">
        <img style="position: absolute; left: 15%; top: 4%" src="../assets/icons/sarge.svg" />
        <h2 style="position: absolute; right: 15%; top: 4%">SARGE LAUNCHER</h2>
        <p style="margin-top: 72px">
          Sarge Launcher is a utility for Quake 3 Arena and Quake 3 Arena mods that provides some useful features for both n00bs and
          veterans. If you are new, you should purchase the game first, then update your client to ioquake3 or quake3e.
        </p>
        <p>Once you have Q3A you can link the client or mod client using +</p>
        <p>
          Sarge Launcher is not responsible for installing any mods, modifying any configs, or downloading any content for Quake 3 Arena -
          YOU are solely responsible for that!
        </p>
        <p style="text-align: center;">
          v0.0.1 copyright n shit
        </p>
      </div>
    </Modal>
  </Teleport>

  <div class="client-grid" draggable="false">
    <div
      class="grid-bg welcome-bg grow"
      @mouseover="hoveredCard = 'welcome'"
      @mouseleave="hoveredCard = ''"
      @click="config.welcome_message = true"
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
      <a class="link" href="https://www.youtube.com/watch?v=wC10pyS0Gyk&list=PLGGojOY6nta5NmPMshZE9l5y3WOaQtU7O&index=1" target="_blank">
        <img src="../assets/images/contenders.png" class="q3-video" />
        <div v-if="hoveredCard == 'quake 3 arena'" class="center card-name tint">
          <span class="center">{{ hoveredCard }}</span>
        </div>
      </a>
    </div>
    <div
      class="grid-bg plus-bg grow"
      @mouseover="hoveredCard = 'add client'"
      @mouseleave="hoveredCard = ''"
      style="cursor: pointer; background-color: var(--secondary-bg); grid-column: 5; grid-row: 1"
    >
      <div v-if="hoveredCard == 'add client'" class="tint" @click="pickQ3Client()">
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
        </div>
      </a>
    </div>

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
    height: 100%;
    display: grid;
    grid-template-columns: repeat(5, minmax(111px, 20%));
    grid-template-rows: repeat(4, minmax(74px, 25%));
    grid-gap: 16px;

    padding: 4px;
    color: white;
    user-select: none;
  }

  .grid-bg {
    background-color: var(--secondary-bg);
    background-repeat: no-repeat;
    background-size: 100%;
    background-position: center center;
    color: white;
  }

  .plus-bg {
    background-image: url('../assets/icons/plus.svg');
    background-size: 10%;
  }

  .banner-bg {
    background-image: url('../assets/images/code.png');
    background-size: 100%;
    background-position: 20% 40%;
  }

  .quake-bg {
    background-image: url('../assets/images/q3_box_art.jpg');
  }

  .welcome-bg {
    background-image: url('../assets/icons/sarge.svg');
    background-size: 40%;
  }

  .upgrade-bg {
    background-image: url('../assets/icons/q3-white.svg');
    background-size: 40%;
  }

  .urt-bg {
    background-image: url('../assets/images/q3ut4.png');
    background-size: 60%;
  }

  .defrag-bg {
    background-image: url('../assets/images/defrag.svg');
    background-size: 60%;
  }

  .cpma-bg {
    background-image: url('../assets/images/cpma.png');
    background-size: 60%;
  }

  .map-bg {
    background-image: url('../assets/images/map-nightmare.jpg');
    background-position: center center;
  }

  .mod-bg {
    background-position: center center;
  }

  .oa-bg {
    background-image: url('../assets/images/baseoa.svg');
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
  }
</style>
