import ClientDashboard from '@/views/ClientDashboard.vue'
import Resource from '@/views/Resource.vue'
import SinglePLayer from '@/views/SinglePlayer.vue'
import Server from '@/views/Server.vue'
import Demo from '@/views/Demo.vue'
import { createRouter, createWebHashHistory } from 'vue-router'
import { type RouteRecordRaw } from 'vue-router'

const routes: RouteRecordRaw[] = [
  {
    path: '/',
    redirect: '/clientdash',
  },
  {
    path: '/clientdash',
    name: 'clientdash',
    component: ClientDashboard,
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
