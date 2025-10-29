<script setup lang="ts">
  import { ref, onMounted, onBeforeUnmount, computed } from 'vue'
  import ClientProfile from '@/components/ClientProfile.vue'
  import Modal from '@/components/Modal.vue'
  import { type Q3Executable } from '@/models/client'
  import { useConfig } from '@/composables/config'
  import { useClient } from '@/composables/client'

  const { config,  } = useConfig()
  const { activeClient, toggleQ3Client, deleteQ3Client, getClientDefaultGamename } = useClient()

  const isDropDownVisible = ref(false)
  const settingsHovered = ref(false)
  const settingsClicked = ref(false)

  const activeClientNameOrDefault = computed(() => {
    if (!activeClient.value) { return 'Quake 3 Client'}
    return removeDotSuffix(activeClient.value.name)
  })

  function removeDotSuffix(name: string) {
    let dot = name.indexOf('.')
    let withoutSuffix =  name.substring(0, dot == -1 ? name.length : dot)
    return withoutSuffix
  }

  function toggleClientSelect(client: Q3Executable) {
    if (settingsClicked.value) {
      settingsClicked.value = false
      return
    }
    toggleQ3Client(client)
  }

  function deleteClient(client: Q3Executable) {
    settingsClicked.value = true
    deleteQ3Client(client)
    isDropDownVisible.value = false
    settingsHovered.value = false
  }

  const showClientProfile = ref(false)
  const profiledClient = ref<Q3Executable | null>(null)

  function showProfile(client: Q3Executable) {
    settingsClicked.value = true
    showClientProfile.value = true
    isDropDownVisible.value = false
    settingsHovered.value = false

    profiledClient.value = client
  }

  function handleKeyPress(event: KeyboardEvent) {
    if (event.code == 'Escape') {
      isDropDownVisible.value = false
    }
  }

  function clientIsOverridden() {
    if (!activeClient.value) { return false }
    if (activeClient.value.gamename != getClientDefaultGamename(activeClient.value)) {
      return true
    }
    return false
  }

  onMounted(() => {
    document.addEventListener('keydown', handleKeyPress)
  })
  onBeforeUnmount(() => {
    document.removeEventListener('keydown', handleKeyPress)
  })
</script>

<template>
  <div
    class="active-client"
    :class="activeClient ? 'not-empty' : 'empty'"
    :style="isDropDownVisible ? 'background-color: var(--alt-bg);' : ''"
    id="dropdown"
    @click.prevent="isDropDownVisible = activeClient ? !isDropDownVisible : false"
  >

    {{ activeClientNameOrDefault }}
    <span v-if="clientIsOverridden()" class="overridden">{{ activeClient?.gamename }}</span>

    <div class="clients-wrapper" v-if="isDropDownVisible">
      <div
        v-for="(client, index) in config.q3_clients"
        class="client"
        :class="client.gamename"
        :key="client.exe_path"
        :style="settingsHovered ? 'background-color: var(--main-bg);' : ''"
        :id="client.exe_path === activeClient?.exe_path ? 'selected-client': `client-${index}`"
        @click.prevent="toggleClientSelect(client)"
      >
        <button
          class="settings-button"
          @mouseover="settingsHovered = true"
          @mouseleave="settingsHovered = false"
          @click.prevent="showProfile(client)"
        >
          <img src="../assets/icons/settings.svg" width="16px" />
        </button>

        <span>{{ removeDotSuffix(client.name) }}</span>
        
      </div>
    </div>
    <Teleport to="#modal">
      <Modal v-if="showClientProfile" :popup-type="'center'" @cancelModal="showClientProfile=false" @executeModal="showClientProfile=false">
        <ClientProfile :profiledClient="profiledClient!" @deleteClient="showClientProfile=false; deleteClient(profiledClient!); isDropDownVisible = activeClient ? !isDropDownVisible : false"/>
      </Modal>
    </Teleport>
  </div>
</template>

<style scoped>

  #selected-client {
    font-weight: 800;
  }

  .active-client {
    padding: 2px;
    width: 160px;
    background-color: var(--main-bg);
    margin: 2px 4px 4px 0px;
    border-radius: 0.2rem;
    height: 24px;
    line-height: 24px;
    white-space: nowrap;
    overflow: hidden;
  }

  .empty {
    cursor: default;
    background-color: var(--secondary-bg);
    border: solid 1px var(--main-bg);
    color: var(--main-bg);
  }

  .not-empty {
    cursor: pointer;
    color: white;
  }

  .not-empty:hover {
    background-color: var(--main-bg);
  }

  .overridden {
    font-size: 70%; 
    color: orange; 
    font-weight: 800;
  }

  .clients-wrapper {
    position: fixed;
    top: calc(3vh + 44px);
    right: 22px;
    border-radius: 0.2rem;
    width: 248px;
    background: var(--main-bg);
    box-shadow: 0px 0px 8px var(--secondary-bg);
    overflow: auto;
    max-height: 294px;
  }

  .client {
    display: flex;
    align-items: center;
    line-height: 42px;
    cursor: pointer;
    text-align: left;
    padding-left: 10px;
    overflow: hidden;
  }

  .client:hover {
    background-color: var(--alt-bg);
  }

  .client:first-of-type {
    border-top-left-radius: 0.2rem;
    border-top-right-radius: 0.2rem;
  }

  .client:last-of-type {
    border-bottom-left-radius: 0.2rem;
    border-bottom-right-radius: 0.2rem;
  }

  .defrag {
    background: url('../assets/images/defrag.svg') 90% center no-repeat;
    background-size: 30%;
  }

  .cpma {
    background: url('../assets/images/cpma.png') 90% center no-repeat;
    background-size: 30%;
  }

  .baseoa {
    background: url('../assets/images/baseoa.svg') 90% center no-repeat;
    background-size: 30%;
  }

  .q3ut4 {
    background: url('../assets/images/q3ut4.png') 90% center no-repeat;
    background-size: 28%;
  }

  .baseq3 {
    background: url('../assets/images/baseq3.png') 90% center no-repeat;
    background-size: 30%;
  }

  .osp {
    background: url('../assets/images/osp.png') 94% center no-repeat;
    background-size: 35%;
  }

  .excessiveplus {
    background: url('../assets/images/excessiveplus.png') 94% center no-repeat;
    background-size: 39%;
  }

  .rat {
    background: url('../assets/images/rat.png') 85% center no-repeat;
    background-size: 20%;
  }

  .settings-button {
    background: var(--main-bg);
    border: 1px solid #777;
    border-radius: 0.2rem;
    margin-right: 8px;
    padding-inline-start: 2px;
    padding-inline-end: 3px;
    padding-block-start: 2px;
    padding-block-end: 0px;
  }

  .settings-button:hover {
    background: var(--alt-bg);
    cursor: pointer;
  }

</style>
