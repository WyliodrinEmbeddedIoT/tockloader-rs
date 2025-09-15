import { createRouter, createWebHistory } from 'vue-router'
import HomeView from '../views/HomeView.vue'
import KnownBoardView from '../views/KnownBoardView.vue'
import CustomizeDebugView from '../views/CustomizeDebugView.vue'
import CustomizeSerialView from '../views/CustomizeSerialView.vue'
import DeviceManagerView from '../views/DeviceManagerView.vue'

const routes = [
  {
    path: '/',
    name: 'Home',
    component: HomeView
  },
  {
    path: '/known-board',
    name: 'KnownBoard',
    component: KnownBoardView
  },
  {
    path: '/customize-debug',
    name: 'CustomizeDebug',
    component: CustomizeDebugView
  },
  {
    path: '/customize-serial',
    name: 'CustomizeSerial',
    component: CustomizeSerialView
  },
  {
    path: '/device-manager',
    name: 'DeviceManager',
    component: DeviceManagerView
  },
]

const router = createRouter({
  history: createWebHistory(),
  routes
})

export default router
