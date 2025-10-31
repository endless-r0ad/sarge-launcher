<script setup lang="ts">
  import { defineProps, defineEmits, onMounted, onBeforeUnmount } from 'vue'

  const props = defineProps<{ popupType: string }>()

  const emit = defineEmits<{ close: [] }>()

  function handleKeyPress(event: KeyboardEvent) {
    if (event.code == 'Escape') {
      emit('close')
    }
  }

  onMounted(() => {
    document.addEventListener('keydown', handleKeyPress)
  })

  onBeforeUnmount(() => {
    document.removeEventListener('keydown', handleKeyPress)
  })
</script>

<template>
  <div class="backdrop" @mousedown.self="emit('close')">
    <div class="modal" :class="props.popupType">
      <span v-if="props.popupType == 'error'">(!)&nbsp;&nbsp;</span>
      <slot></slot>
    </div>
  </div>
</template>

<style scoped>
  .modal {
    border-radius: 0.2rem;
    max-width: max-content;
    max-height: max-content;
    text-align: left;
    z-index: 999;
    border: 1px solid var(--main-bg);
  }

  .center {
    padding: 24px;
    margin: auto;
    background-color: var(--secondary-bg);
    position: fixed;
    inset: 0px;
    overflow: hidden auto;
    color: white;
  }

  .error {
    width: max-content;
    height: max-content;
    padding: 16px;
    margin: 40px auto;
    background-color: #b65718;
    height: 240px;
    overflow: auto;
  }

  .info {
    width: max-content;
    height: max-content;
    padding: 16px;
    margin: 40px auto;
    background-color: #444;
    height: 240px;
  }
</style>
