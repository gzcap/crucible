import { createRouter, createWebHashHistory } from 'vue-router'
import type { RouteRecordRaw } from 'vue-router'
import AppLayout from '../components/layout/AppLayout.vue'
import VaultPicker from '../components/vault/VaultPicker.vue'

const routes: RouteRecordRaw[] = [
  {
    path: '/',
    name: 'Home',
    component: AppLayout,
  },
  {
    path: '/graph',
    name: 'Graph',
    component: AppLayout,
  },
  {
    path: '/vault-picker',
    name: 'VaultPicker',
    component: VaultPicker,
  },
  {
    path: '/:pathMatch(.*)*',
    redirect: '/',
  },
]

const router = createRouter({
  history: createWebHashHistory(),
  routes,
})

export default router
