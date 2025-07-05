<script setup lang="ts">
  import DropDown from '@/components/DropDown.vue';
  import { invoke } from '@tauri-apps/api/core'
  import { defineProps, defineEmits, ref } from 'vue';
  import { ensureError } from '@/utils/util';
  import { type Q3Executable } from '@/models/client';
  import { type Nullable } from '@/utils/util';
  import { type Config } from '@/models/config'

  const props = defineProps<{ currentView: string, config: Config, activeClient: Nullable<Q3Executable> }>();
  
  const emit = defineEmits<{
    spawnQuake: [string],
    addQ3Client: [Q3Executable],
    toggleQ3Client: [Q3Executable],
    deleteQ3Client: [Q3Executable],
    errorAlert: [string]
  }>();

  const componentName = ref('Header');
  
  function spawnQuake() {
    emit('spawnQuake', componentName.value)
  }

  async function pickClientBlocking() {
    try {
      let new_client: Q3Executable = await invoke('pick_client_blocking')

      if (new_client != null) {
        emit('addQ3Client', new_client)
      }    
    }
    catch(err) {
      emit('errorAlert', ensureError(err).message)   
    } 
  }
  function q3ClientSelect(client: Q3Executable) {
    emit('toggleQ3Client', client)
  }

  function deleteClient(client: Q3Executable) {
    emit('deleteQ3Client', client)
  }
    


</script>

<template>

  <div class="navbar"  >
    <div><h3 class="page-title"> {{ currentView }}</h3></div>

    <div class="nav-right">
      <div v-if="props.activeClient" class="launch-client-button" @click="spawnQuake" />
      <div class="add-client-button" @click="pickClientBlocking"></div>

      <DropDown 
        :q3Clients="config.q3_clients"
        :activeClient="activeClient"
        @q3ClientSelect="q3ClientSelect"
        @deleteClient="deleteClient"
      />
      
    </div>   
    
  </div>

  
</template>
    
  
    
    
<style scoped>

  .navbar {
    background-color: var(--secondary-bg);
    height: 38px;
    width: calc(100% - 146px);
    border: solid 1px #222831;
    border-radius: 0.2rem;

    position: absolute;
    top: 3vh;
    left: 112px;
    float: left;
    text-align: center;

    display: flex;
    user-select: none;

    z-index: 998;
    line-height: 38px;
  }

  .navbar div:last-child {
    margin-left: auto;
  }
  
  .page-title {
    color: #fff; 
    padding-left: 12px;
    align-items: center;
    margin: 0 auto;
  }

  .nav-right {
    display: flex; 
    justify-content: right; 
    padding: 2px; 
    line-height: 38px;
  }
  
  .launch-client-button {
    padding: 0px 16px;
    margin: 2px 4px 4px 0px;
    background: url('../assets/icons/play.svg') center center no-repeat;
    
  }

  .launch-client-button:hover {
    background-color: var(--main-bg);;
    border-radius: 0.2rem;
    cursor: pointer;
  }


  .add-client-button {
    padding: 0px 16px;
    margin: 2px 4px 4px 0px;
    background: url('../assets/icons/plus.svg') center center no-repeat;  
    cursor: pointer;
  }

  .add-client-button:hover {
    background-color: var(--main-bg);;
    border-radius: 0.2rem;
  }

</style>