import ClientSetup from '@/views/ClientSetup.vue'
import Resource from '@/views/Resource.vue'
import SinglePLayer from '@/views/SinglePlayer.vue'
import Server from '@/views/Server.vue'
import Demo from '@/views/Demo.vue'
import { createRouter, createWebHashHistory } from 'vue-router'
import { type RouteRecordRaw } from 'vue-router'

const routes: RouteRecordRaw[] = [
  {
    path: '/',
    redirect: '/clientsetup',
  },
  {
    path: '/clientsetup',
    name: 'clientsetup',
    component: ClientSetup,
  },
  {
    path: '/resource',
    name: 'resource',
    component: Resource,
  },
  {
    path: '/singleplayer',
    name: 'singleplayer',
    component: SinglePLayer,
  },
  {
    path: '/server',
    name: 'server',
    component: Server,
  },
  {
    path: '/demo',
    name: 'demo',
    component: Demo,
  },
]

const router = createRouter({
  history: createWebHashHistory(),
  routes: routes,
  linkActiveClass: 'router-active',
})

export default router
