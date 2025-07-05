<script setup lang="ts">
  import { defineProps, defineEmits, onMounted, onBeforeUnmount } from 'vue';

  const props = defineProps<{ popupType: string }>(); // config: Config, appData: AppData,
  
  const emit = defineEmits<{
    cancelModal: [],
    executeModal: [],
    mutateConfig: [],
  }>();

  
  function handleKeyPress(event: KeyboardEvent) {
    if (event.code == 'Escape') {
      emit('cancelModal')
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
  <div class="backdrop" @mousedown.self="emit('cancelModal')" >
      <div :class="props.popupType"> 
        <div>
          <span v-if="props.popupType == 'error'">(!)&nbsp;&nbsp;</span>
          <slot></slot>
        </div>  
      </div>    
  </div>
</template>

<style scoped>
  .backdrop {
    /* top: 52px !important; */
    top: 0;
    left: 0;
    bottom: 0;
    position: fixed;
    background: rgba(0, 0, 0, 0.6);
    width: 100%;
    /* height: 100%; */
    z-index: 999;
    color: white;
  }

  .center-text {
    text-align: center; 
    margin: 4px 1px auto auto;
    
    font-size: 1em;
    text-transform: uppercase;
    letter-spacing: 1px;
    font-weight: bold;
    color: black;
  }

  .center {

    padding: 24px;
    margin:  auto;
    background-color: var(--secondary-bg);
    border-radius: 0.2rem;
    position: fixed;
    inset: 0px;
    max-width: max-content;
    max-height: max-content;
    text-align: left;
    overflow: hidden auto;
    color: white;
    border: 1px solid var(--main-bg);
  }

  .error {

    width: max-content;
    height: max-content;
      /* padding: 10px; */
    padding: 16px;
    margin:  40px auto;
    /* border-radius: 10px; */
    background-color: #b65718;
    /* border: 2px solid #fff; */
    border-radius: 0.2rem;

    /* width: 12rem; */
    height: 240px;
    max-width: max-content;
    max-height: max-content;
    overflow: auto;
    z-index: 999;
    border: 1px solid var(--main-bg);;
    /* border: 1px solid white; */
    }

  .info {

    width: max-content;
    height: max-content;
      /* padding: 10px; */
    padding: 16px;
    margin:  40px auto;
    /* border-radius: 10px; */
    background-color: #444;
    /* border: 2px solid #fff; */
    border-radius: 0.2rem;

    /* width: 12rem; */
    height: 240px;
    max-width: max-content;
    max-height: max-content;
    z-index: 999;
    border: 1px solid var(--main-bg);;
    /* border: 1px solid white; */
    }


</style>