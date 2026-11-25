import { defineStore } from 'pinia'
import { api } from '../api'

export const useAuthStore = defineStore('auth', {
  state: () => ({ user: null, needsSetup: false, ready: false }),
  getters: {
    isManager: (s) => s.user?.role === 'manager',
  },
  actions: {
    async init() {
      this.needsSetup = await api.needsSetup()
      this.user = await api.currentUser()
      this.ready = true
    },
    async login(username, password) {
      this.user = await api.login(username, password)
    },
    async setup(input) {
      this.user = await api.setupFirstManager(input)
      this.needsSetup = false
    },
    async logout() {
      try {
        await api.logout()
      } finally {
        this.user = null
      }
    },
  },
})
