<script setup>
import { ref, reactive, onMounted } from 'vue'
import { api } from '../../api'
import { CARD_TYPE_LABEL, VEHICLE_LABEL } from '../../utils/format'

const cards = ref([])
const error = ref('')
const ok = ref('')

const blank = () => ({
  id: '', display_code: '', card_type: 'guest', vehicle_type: '',
  owner_name: '', owner_phone: '', vehicle_plate: '', valid_to: '',
})
const form = reactive(blank())

async function load() {
  try {
    cards.value = await api.listCards()
  } catch (e) {
    error.value = e.message
  }
}

async function submit() {
  error.value = ''
  ok.value = ''
  try {
    const c = await api.createCard({ ...form })
    ok.value = `Đã thêm thẻ ${c.display_code}`
    Object.assign(form, blank())
    await load()
  } catch (e) {
    error.value = e.message
  }
}

onMounted(load)
</script>

<template>
  <h1>Quản lý thẻ</h1>

  <form class="panel" style="margin-bottom: 20px" @submit.prevent="submit">
    <div v-if="error" class="alert alert-error">{{ error }}</div>
    <div v-if="ok" class="alert alert-success">{{ ok }}</div>

    <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 12px">
      <div class="field">
        <label for="c-id">Mã thẻ (hex từ đầu đọc)</label>
        <input id="c-id" v-model="form.id" required />
      </div>
      <div class="field">
        <label for="c-code">Mã in trên thẻ</label>
        <input id="c-code" v-model="form.display_code" required />
      </div>
      <div class="field">
        <label for="c-type">Loại thẻ</label>
        <select id="c-type" v-model="form.card_type">
          <option v-for="(label, k) in CARD_TYPE_LABEL" :key="k" :value="k">{{ label }}</option>
        </select>
      </div>
      <div class="field">
        <label for="c-vehicle">Loại xe</label>
        <select id="c-vehicle" v-model="form.vehicle_type" :required="form.card_type === 'monthly'">
          <option value="">(theo làn)</option>
          <option v-for="(label, k) in VEHICLE_LABEL" :key="k" :value="k">{{ label }}</option>
        </select>
      </div>
      <div class="field">
        <label for="c-owner">Chủ thẻ</label>
        <input id="c-owner" v-model="form.owner_name" />
      </div>
      <div class="field">
        <label for="c-phone">SĐT</label>
        <input id="c-phone" v-model="form.owner_phone" />
      </div>
      <div class="field">
        <label for="c-plate">Biển số</label>
        <input id="c-plate" v-model="form.vehicle_plate" />
      </div>
      <div class="field">
        <label for="c-valid">Hạn đến</label>
        <input id="c-valid" v-model="form.valid_to" type="date" />
      </div>
    </div>
    <button class="btn">Thêm thẻ</button>
  </form>

  <table>
    <thead>
      <tr><th>Mã in</th><th>Mã hex</th><th>Loại</th><th>Chủ thẻ</th><th>Biển số</th><th>Hạn</th><th>Trạng thái</th></tr>
    </thead>
    <tbody>
      <tr v-for="c in cards" :key="c.id">
        <td>{{ c.display_code }}</td>
        <td>{{ c.id }}</td>
        <td>{{ CARD_TYPE_LABEL[c.card_type] }}</td>
        <td>{{ c.owner_name }}</td>
        <td>{{ c.vehicle_plate }}</td>
        <td>{{ c.valid_to?.slice(0, 10) }}</td>
        <td>{{ c.status }}</td>
      </tr>
      <tr v-if="!cards.length"><td colspan="7">Chưa có thẻ nào.</td></tr>
    </tbody>
  </table>
</template>
