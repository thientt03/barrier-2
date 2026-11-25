<script setup>
import { useRouter } from 'vue-router'
import { useAuthStore } from './stores/auth'

const auth = useAuthStore()
const router = useRouter()

async function logout() {
  await auth.logout()
  router.push({ name: 'login' })
}
</script>

<template>
  <nav v-if="auth.user" class="topbar">
    <strong class="brand">Bãi đỗ xe</strong>
    <router-link to="/entry">Xe vào</router-link>
    <router-link to="/exit">Xe ra</router-link>
    <router-link v-if="auth.isManager" to="/cards">Thẻ</router-link>
    <router-link v-if="auth.isManager" to="/settings">Cài đặt</router-link>
    <span class="spacer" />
    <span class="who">{{ auth.user.full_name }} ({{ auth.user.role === 'manager' ? 'Quản lý' : 'Bảo vệ' }})</span>
    <button class="btn btn-secondary" @click="logout">Đăng xuất</button>
  </nav>
  <main>
    <router-view />
  </main>
</template>
