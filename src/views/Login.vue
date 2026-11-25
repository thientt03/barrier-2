<script setup>
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '../stores/auth'

const auth = useAuthStore()
const router = useRouter()

const username = ref('')
const password = ref('')
const fullName = ref('')
const error = ref('')
const busy = ref(false)

async function submit() {
  error.value = ''
  busy.value = true
  try {
    if (auth.needsSetup) {
      await auth.setup({ username: username.value, password: password.value, full_name: fullName.value })
    } else {
      await auth.login(username.value, password.value)
    }
    router.push({ name: 'entry' })
  } catch (e) {
    error.value = e.message
  } finally {
    busy.value = false
  }
}
</script>

<template>
  <form class="panel login-box" @submit.prevent="submit">
    <h1>{{ auth.needsSetup ? 'Thiết lập lần đầu' : 'Đăng nhập' }}</h1>
    <p v-if="auth.needsSetup" class="hint">Tạo tài khoản quản lý đầu tiên cho hệ thống.</p>

    <div v-if="error" class="alert alert-error">{{ error }}</div>

    <div v-if="auth.needsSetup" class="field">
      <label for="full-name">Họ tên</label>
      <input id="full-name" v-model="fullName" autocomplete="name" required />
    </div>
    <div class="field">
      <label for="username">Tên đăng nhập</label>
      <input id="username" v-model="username" autocomplete="username" required autofocus />
    </div>
    <div class="field">
      <label for="password">Mật khẩu</label>
      <input
        id="password"
        v-model="password"
        type="password"
        :autocomplete="auth.needsSetup ? 'new-password' : 'current-password'"
        required
      />
      <small v-if="auth.needsSetup" class="hint">Tối thiểu 8 ký tự.</small>
    </div>
    <button class="btn" :disabled="busy" style="width: 100%">
      {{ auth.needsSetup ? 'Tạo tài khoản' : 'Đăng nhập' }}
    </button>
  </form>
</template>
