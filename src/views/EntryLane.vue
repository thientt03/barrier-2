<script setup>
import { ref, computed, onMounted, onUnmounted, nextTick } from 'vue'
import { api } from '../api'
import { listen } from '@tauri-apps/api/event'
import PhotoCard from '../components/PhotoCard.vue'
import CardInfo from '../components/CardInfo.vue'
import { timeOnly } from '../utils/format'

const laneKind = ref('car') // 'car' | 'bike'
const lane = computed(() => `entry_${laneKind.value}`)

const code = ref('')
const preview = ref(null)
const error = ref('')
const done = ref('')
const busy = ref(false)
const codeInput = ref(null)

const focusCode = () => nextTick(() => codeInput.value?.focus())

// Đầu đọc USB HID gõ mã thẻ vào ô này rồi gửi Enter; cũng dùng để nhập tay (F2).
async function scan() {
  const value = code.value.trim()
  code.value = ''
  if (!value || busy.value) return
  error.value = ''
  done.value = ''
  busy.value = true
  try {
    preview.value = await api.prepareEntry(value, lane.value)
  } catch (e) {
    preview.value = null
    error.value = e.message
  } finally {
    busy.value = false
    focusCode()
  }
}

async function confirm() {
  if (!preview.value || busy.value) return
  busy.value = true
  try {
    await api.confirmEntry(lane.value)
    done.value = `Đã ghi nhận xe vào — thẻ ${preview.value.card.display_code}. Bấm nút vật lý để mở barrier.`
    preview.value = null
  } catch (e) {
    error.value = e.message
    preview.value = null
  } finally {
    busy.value = false
    focusCode()
  }
}

async function reject() {
  if (preview.value) await api.cancelPending(lane.value).catch(() => {})
  preview.value = null
  error.value = ''
  focusCode()
}

async function recapture() {
  if (!preview.value || busy.value) return
  try {
    preview.value.photo = await api.recapture(lane.value)
  } catch (e) {
    error.value = e.message
  }
}

async function switchLane(kind) {
  if (kind === laneKind.value) return
  await reject()
  laneKind.value = kind
  done.value = ''
}

function onKeydown(e) {
  if (e.key === 'Enter') {
    if (code.value.trim()) scan()
    else if (preview.value) confirm()
  } else if (e.key === 'Escape') {
    reject()
  } else if (e.key === 'F2') {
    e.preventDefault()
    codeInput.value?.focus()
  } else if (e.key === 'F5') {
    e.preventDefault()
    recapture()
  }
}

let unlisten
onMounted(async () => {
  window.addEventListener('keydown', onKeydown)
  focusCode()
  // Đầu đọc cổng serial (cấu hình ở Cài đặt) gửi mã thẻ qua sự kiện này.
  unlisten = await listen('card-tapped', (e) => {
    if (e.payload.direction !== 'entry') return
    code.value = e.payload.card_id
    scan()
  })
})
onUnmounted(() => {
  window.removeEventListener('keydown', onKeydown)
  unlisten?.()
  if (preview.value) api.cancelPending(lane.value).catch(() => {})
})
</script>

<template>
  <h1>BỐT VÀO — LÀN {{ laneKind === 'car' ? 'Ô TÔ' : 'XE MÁY' }}</h1>

  <div class="lane-tabs">
    <button class="btn" :class="{ 'btn-secondary': laneKind !== 'car' }" @click="switchLane('car')">Ô tô</button>
    <button class="btn" :class="{ 'btn-secondary': laneKind !== 'bike' }" @click="switchLane('bike')">Xe máy</button>
  </div>

  <div class="scan">
    <input
      ref="codeInput"
      v-model="code"
      placeholder="Quẹt thẻ hoặc nhập mã thẻ (F2)"
      autocomplete="off"
      spellcheck="false"
    />
  </div>

  <div v-if="error" class="alert alert-error">⚠️ {{ error }}</div>
  <div v-if="done" class="alert alert-success">✅ {{ done }}</div>

  <div v-if="preview" class="panel split">
    <div class="photos">
      <PhotoCard :src="preview.photo" :caption="`Ảnh xe vào — ${timeOnly(preview.entry_time)}`" />
    </div>
    <div>
      <CardInfo :card="preview.card" />
      <div class="actions">
        <button class="btn btn-secondary" @click="recapture">📸 Chụp lại (F5)</button>
        <button class="btn btn-danger" @click="reject">❌ Hủy (Esc)</button>
        <button class="btn btn-success" :disabled="busy" @click="confirm">✅ XÁC NHẬN VÀO (Enter)</button>
      </div>
      <p class="hint">💡 Sau khi xác nhận: bấm nút vật lý để mở barrier.</p>
    </div>
  </div>
  <div v-else class="waiting">Đang chờ quẹt thẻ...</div>
</template>
