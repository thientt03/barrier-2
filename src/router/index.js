import { createRouter, createWebHashHistory } from 'vue-router'
import { useAuthStore } from '../stores/auth'
import { setUnauthenticatedHandler } from '../api'

const routes = [
  { path: '/login', name: 'login', component: () => import('../views/Login.vue') },
  { path: '/', redirect: '/entry' },
  { path: '/entry', name: 'entry', component: () => import('../views/EntryLane.vue'), meta: { auth: true } },
  { path: '/exit', name: 'exit', component: () => import('../views/ExitLane.vue'), meta: { auth: true } },
  { path: '/cards', name: 'cards', component: () => import('../views/manager/Cards.vue'), meta: { auth: true, manager: true } },
  { path: '/settings', name: 'settings', component: () => import('../views/manager/Settings.vue'), meta: { auth: true, manager: true } },
  { path: '/forbidden', name: 'forbidden', component: () => import('../views/common/Forbidden.vue'), meta: { auth: true } },
  { path: '/:pathMatch(.*)*', name: 'not-found', component: () => import('../views/common/NotFound.vue') },
]

const router = createRouter({ history: createWebHashHistory(), routes })

router.beforeEach(async (to) => {
  const auth = useAuthStore()
  if (!auth.ready) await auth.init()
  if (to.meta.auth && !auth.user) return { name: 'login' }
  if (to.meta.manager && !auth.isManager) return { name: 'forbidden' }
  if (to.name === 'login' && auth.user) return { name: 'entry' }
})

setUnauthenticatedHandler(() => {
  useAuthStore().user = null
  router.push({ name: 'login' })
})

export default router
