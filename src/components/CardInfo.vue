<script setup>
import { CARD_TYPE_LABEL, VEHICLE_LABEL, isVip } from '../utils/format'

defineProps({ card: { type: Object, required: true } })
</script>

<template>
  <dl class="info-grid">
    <dt>Thẻ</dt>
    <dd>
      {{ card.display_code }}
      <span class="badge" :class="`badge-${card.card_type}`">{{ CARD_TYPE_LABEL[card.card_type] }}</span>
      <span v-if="isVip(card)" class="badge badge-vip">⭐ VIP</span>
    </dd>
    <template v-if="card.owner_name">
      <dt>Chủ thẻ</dt>
      <dd>{{ card.owner_name }}<template v-if="card.owner_phone"> — {{ card.owner_phone }}</template></dd>
    </template>
    <template v-if="card.vehicle_plate">
      <dt>Biển số</dt>
      <dd>{{ card.vehicle_plate }}</dd>
    </template>
    <template v-if="card.vehicle_type">
      <dt>Loại xe</dt>
      <dd>{{ VEHICLE_LABEL[card.vehicle_type] }}</dd>
    </template>
    <template v-if="card.notes">
      <dt>Ghi chú</dt>
      <dd>{{ card.notes }}</dd>
    </template>
    <template v-if="card.valid_to">
      <dt>Hạn</dt>
      <dd>{{ card.valid_to.slice(0, 10) }}</dd>
    </template>
  </dl>
</template>
