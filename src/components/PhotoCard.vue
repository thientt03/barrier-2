<script setup>
defineProps({ src: String, caption: String })
</script>

<template>
  <figure class="photo-card">
    <img v-if="src" :src="src" :alt="caption" />
    <div v-else class="missing">Không có ảnh</div>
    <figcaption>{{ caption }}</figcaption>
  </figure>
</template>
