<script setup>
import { ref, computed, onMounted, onUnmounted, nextTick } from 'vue'
import { api } from '../api'
import { listen } from '@tauri-apps/api/event'
import PhotoCard from '../components/PhotoCard.vue'
import CardInfo from '../components/CardInfo.vue'
import { formatDuration, formatMoney, timeOnly } from '../utils/format'

const laneKind = ref('car')
const lane = computed(() => `exit_${laneKind.value}`)

const code = ref('')
const preview = ref(null)
const error = ref('')
const done = ref('')
const busy = ref(false)
const codeInput = ref(null)

const focusCode = () => nextTick(() => codeInput.value?.focus())

async function scan() {
  const value = code.value.trim()
  code.value = ''
  if (!value || busy.value) return
  error.value = ''
  done.value = ''
  busy.value = true
  try {
    preview.value = await api.prepareExit(value, lane.value)
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
    const fee = await api.confirmExit(lane.value)
    done.value =
      fee > 0
        ? `Đã thu ${formatMoney(fee)}. Bấm nút vật lý để mở barrier.`
        : 'Đã ghi nhận xe ra (miễn phí). Bấm nút vật lý để mở barrier.'
    preview.value = null
  } catch (e) {
    error.value = e.message
    preview.value = null
  } finally {
    busy.value = false
    focusCode()
  }
}

// Từ chối cho xe ra: phiên vẫn đang đỗ, ghi audit refuse_exit.
async function refuse() {
  if (!preview.value || busy.value) return
  busy.value = true
  try {
    await api.refuseExit(lane.value)
    done.value = 'Đã từ chối cho xe ra. KHÔNG mở barrier.'
    preview.value = null
  } catch (e) {
    error.value = e.message
  } finally {
    busy.value = false
    focusCode()
  }
}

async function recapture() {
  if (!preview.value || busy.value) return
  try {
    preview.value.exit_photo = await api.recapture(lane.value)
  } catch (e) {
    error.value = e.message
  }
}

async function switchLane(kind) {
  if (kind === laneKind.value) return
  if (preview.value) await api.cancelPending(lane.value).catch(() => {})
  preview.value = null
  error.value = ''
  done.value = ''
  laneKind.value = kind
}

function onKeydown(e) {
  if (e.key === 'Enter') {
    if (code.value.trim()) scan()
    else if (preview.value) confirm()
  } else if (e.key === 'Escape') {
    refuse()
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
    if (e.payload.direction !== 'exit') return
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
  <h1>BỐT RA — LÀN {{ laneKind === 'car' ? 'Ô TÔ' : 'XE MÁY' }}</h1>

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

  <div v-if="preview" class="panel">
    <div class="photos compact">
      <PhotoCard :src="preview.entry_photo" :caption="`Ảnh lúc vào — ${timeOnly(preview.entry_time)}`" />
      <PhotoCard :src="preview.exit_photo" :caption="`Ảnh lúc ra — ${timeOnly(preview.exit_time)}`" />
    </div>
    <div class="alert alert-warning">⚠️ SO 2 ẢNH — CÓ PHẢI CÙNG 1 XE KHÔNG?</div>
    <div class="split">
      <div>
        <CardInfo :card="preview.card" />
        <p>⏱️ Thời gian đỗ: <strong>{{ formatDuration(preview.duration_minutes) }}</strong></p>
        <div class="fee" :class="{ free: preview.is_free }">
          💰 {{ preview.is_free ? 'Miễn phí' : formatMoney(preview.fee) }}
        </div>
      </div>
      <div>
        <div class="actions stack">
          <button class="btn btn-success" :disabled="busy" @click="confirm">
            ✅ {{ preview.is_free ? 'XÁC NHẬN RA' : 'ĐÃ THU TIỀN' }} (Enter)
          </button>
          <button class="btn btn-danger" :disabled="busy" @click="refuse">❌ TỪ CHỐI (Esc)</button>
          <button class="btn btn-secondary" @click="recapture">📸 Chụp lại (F5)</button>
        </div>
        <p class="hint">💡 Sau khi xác nhận: bấm nút vật lý để mở barrier.</p>
      </div>
    </div>
  </div>
  <div v-else class="waiting">Đang chờ quẹt thẻ...</div>
</template>
