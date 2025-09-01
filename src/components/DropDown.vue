<script setup lang="ts">
  import { ref, onMounted, onBeforeUnmount } from 'vue'
  import { type Q3Executable } from '@/models/client'
  import { useConfig } from '@/composables/config'

  const { config, activeClient, toggleQ3Client, deleteQ3Client } = useConfig();

  const isDropDownVisible = ref(false)
  const deleteHovered = ref(false)
  const deleteClicked = ref(false)

  function toggleClientSelect(client: Q3Executable) {
    if (deleteClicked.value) {
      deleteClicked.value = false
      return
    }
    toggleQ3Client(client)
  }

  function deleteClient(client: Q3Executable) {
    deleteClicked.value = true
    deleteQ3Client(client)
    isDropDownVisible.value = false
  }

  function handleKeyPress(event: KeyboardEvent) {
    if (event.code == 'Escape') {
      isDropDownVisible.value = false
    }
  }

  function handleClick(event: MouseEvent) {
    const target = event.target as HTMLTextAreaElement
    isDropDownVisible.value = target.id == 'dropdown' && activeClient ? !isDropDownVisible.value : false
  }

  onMounted(() => {
    document.addEventListener('keydown', handleKeyPress)
    document.addEventListener('click', handleClick)
  })
  onBeforeUnmount(() => {
    document.removeEventListener('keydown', handleKeyPress)
    document.removeEventListener('click', handleClick)
  })
</script>

<template>
  <div
    class="active-client"
    :class="activeClient ? 'not-empty' : 'empty'"
    :style="isDropDownVisible ? 'background-color: var(--alt-bg);' : ''"
    id="dropdown"
  >
    <img v-if="activeClient?.gamename == 'cpma'" src="../assets/images/cpma.png" class="client-icon-h" style="margin-bottom: -6px;"/>
    <img v-if="activeClient?.gamename == 'defrag'" src="../assets/images/defrag.svg" class="client-icon-h" style="margin-bottom: -8px;"/>
    <img v-if="activeClient?.gamename == 'q3ut4'" src="../assets/images/q3ut4.png" class="client-icon-h" style="margin-bottom: -8px;"/>
    <img v-if="activeClient?.gamename == 'baseq3'" src="../assets/images/contenders.png" class="client-icon-h" style="margin-bottom: -6px;"/>
    <img v-if="activeClient?.gamename == 'baseoa'" src="../assets/images/baseoa.svg" class="client-icon-h" style="margin-bottom: -6px;"/>
    {{ activeClient?.name || 'Quake 3 Client' }}

    <div class="clients-wrapper" v-if="isDropDownVisible">
      <div
        v-for="(client, _index) in config.q3_clients"
        class="client"
        :key="client.exe_path"
        :style="deleteHovered ? 'background-color: var(--main-bg);' : ''"
        @click.prevent="toggleClientSelect(client)"
      >
        <button
          class="delete-button"
          @mouseover="deleteHovered = true"
          @mouseleave="deleteHovered = false"
          @click.prevent="deleteClient(client)"
        >
          <img src="../assets/icons/x.svg" width="8px" />
        </button>
        <img v-if="client.gamename == 'cpma'" src="../assets/images/cpma.png" class="client-icon" style="margin-bottom: -6px;"/>
        <img v-if="client.gamename == 'defrag'" src="../assets/images/defrag.svg" class="client-icon" style="margin-bottom: -8px;"/>
        <img v-if="client.gamename == 'q3ut4'" src="../assets/images/q3ut4.png" class="client-icon" style="margin-bottom: -8px;"/>
        <img v-if="client.gamename == 'baseq3'" src="../assets/images/contenders.png" class="client-icon" style="margin-bottom: -6px;"/>
        <img v-if="client.gamename == 'baseoa'" src="../assets/images/baseoa.svg" class="client-icon" style="margin-bottom: -6px;"/>
        <span>{{ client.name }}</span>
        
      </div>
    </div>
  </div>
</template>

<style scoped>
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

  .clients-wrapper {
    position: fixed;
    top: calc(3vh + 44px);
    right: 22px;
    border-radius: 0.2rem;
    width: 248px;
    background: var(--main-bg);
    box-shadow: 0px 0px 8px var(--secondary-bg);
    overflow: auto;
    max-height: 210px;
  }

  .client {
    display: block;
    line-height: 42px;
    cursor: pointer;
    text-align: left;
    padding-left: 10px;
    overflow: hidden;
  }

  .client:hover {
    background: var(--alt-bg);
  }

  .client:first-of-type {
    border-top-left-radius: 0.2rem;
    border-top-right-radius: 0.2rem;
  }

  .client:last-of-type {
    border-bottom-left-radius: 0.2rem;
    border-bottom-right-radius: 0.2rem;
  }

  .remove-client-button {
    color: white;
    cursor: pointer;
    width: 30px;
    text-align: left;
    background: url('../assets/icons/x.svg') center center no-repeat;
    background-size: 10px;
    display: inline-block;
  }

  .remove-client-button:hover {
    background-color: var(--secondary-bg);
  }

  .delete-button {
    background: var(--main-bg);
    border: 1px solid #777;
    border-radius: 0.2rem;
    margin: 0px 8px 8px 0px;
  }

  .delete-button:hover {
    background: var(--alt-bg);
    cursor: pointer;
  }

  .client-icon-h {
    width: 24px;
    margin-left: -24px;
  }

  .client-icon {
    width: 24px;
    margin-right: 8px;
  }
</style>
