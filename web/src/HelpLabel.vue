<script setup lang="ts">
import { ref, useId } from "vue";
defineProps<{ label: string; help: string; forId?: string }>();
const open = ref(false);
const id = useId();
</script>

<template>
  <span class="field-label">
    <span
      class="help-wrap"
      @keydown.esc="open = false"
      @focusout="open = false"
      @mouseenter="open = true"
      @mouseleave="open = false"
    >
      <button
        type="button"
        class="help-button"
        :aria-label="label"
        :aria-describedby="id"
        :aria-expanded="open"
        :aria-controls="id"
        @focus="open = true"
        @click="open = true"
      >
        ?
      </button>
      <span :id="id" role="tooltip" :class="['help-text', { opened: open }]">{{
        help
      }}</span>
    </span>
    <label v-if="forId" :for="forId">{{ label }}</label>
    <span v-else>{{ label }}</span>
  </span>
</template>
