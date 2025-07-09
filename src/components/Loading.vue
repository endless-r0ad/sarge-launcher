<script setup lang="ts">
  import { defineProps } from 'vue'

  const props = defineProps<{ position: string; message?: string; size: number }>()
</script>

<template>
  <div style="z-index: 999">
    <img :class="props.position" src="../assets/icons/q3-accent.svg" :style="`height: ${props.size}px; width: ${props.size}px`" />
    <div class="center-msg">
      <h2>{{ props.message }}</h2>
    </div>
  </div>
</template>

<style scoped>
  .center-msg {
    position: absolute;
    left: 50%;
    top: 50%;
    transform: translate(-50%, -50%);
    margin-top: 72px;
    color: white;
  }

  .center {
    position: absolute;
    top: 50%;
    left: 50%;
    margin: -50px 0 0 -50px;
    animation: l5 0.75s linear infinite;
  }

  .relative {
    position: relative;
    animation: l5 0.75s linear infinite;
  }

  @keyframes l5 {
    50% {
      transform: rotate(360deg);
    }
    100% {
      transform: rotate(360deg);
    }
  }
</style>
