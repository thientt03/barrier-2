<script setup>
import { ref, reactive, onMounted, onUnmounted } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { api } from '../../api'

const BAUDS = [2400, 4800, 9600, 19200, 38400, 115200]

const ports = ref([])
const cfg = reactive({ entry_port: '', exit_port: '', baud: 9600 })
const status = reactive({ entry: null, exit: null })
const error = ref('')
const ok = ref('')

async function loadPorts() {
  try {
    ports.value = await api.listSerialPorts()
  } catch (e) {
    error.value = e.message
  }
}

async function save() {
  error.value = ''
  ok.value = ''
  try {
    await api.setReaderConfig({ ...cfg })
    ok.value = 'Đã lưu. Đầu đọc sẽ được mở lại.'
  } catch (e) {
    error.value = e.message
  }
}

let unlisten
onMounted(async () => {
  try {
    Object.assign(cfg, await api.getReaderConfig())
  } catch (e) {
    error.value = e.message
  }
  await loadPorts()
  unlisten = await listen('reader-status', (e) => {
    status[e.payload.direction] = e.payload.connected
  })
})
onUnmounted(() => unlisten?.())

const label = (s) => (s === null ? '' : s ? '🟢 đã kết nối' : '🔴 mất kết nối')
</script>

<template>
  <h1>Cài đặt đầu đọc thẻ</h1>
  <form class="panel" @submit.prevent="save">
    <p class="hint">
      Chỉ cần cấu hình nếu đầu đọc nối qua cổng COM / USB-serial. Đầu đọc USB HID (giả lập bàn phím)
      không cần cấu hình: cứ để trống và quẹt thẻ ở màn hình Xe vào / Xe ra.
    </p>
    <div v-if="error" class="alert alert-error">{{ error }}</div>
    <div v-if="ok" class="alert alert-success">{{ ok }}</div>

    <div class="field">
      <label for="p-entry">Cổng làn VÀO {{ label(status.entry) }}</label>
      <select id="p-entry" v-model="cfg.entry_port">
        <option value="">(không dùng)</option>
        <option v-for="p in ports" :key="p" :value="p">{{ p }}</option>
      </select>
    </div>
    <div class="field">
      <label for="p-exit">Cổng làn RA {{ label(status.exit) }}</label>
      <select id="p-exit" v-model="cfg.exit_port">
        <option value="">(không dùng)</option>
        <option v-for="p in ports" :key="p" :value="p">{{ p }}</option>
      </select>
    </div>
    <div class="field">
      <label for="p-baud">Tốc độ (baud)</label>
      <select id="p-baud" v-model.number="cfg.baud">
        <option v-for="b in BAUDS" :key="b" :value="b">{{ b }}</option>
      </select>
    </div>
    <div class="actions">
      <button type="button" class="btn btn-secondary" @click="loadPorts">Quét lại cổng</button>
      <button class="btn">Lưu</button>
    </div>
  </form>
</template>
