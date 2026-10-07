<script setup lang="ts">
import { computed, nextTick, ref, useId } from "vue";
import HelpLabel from "./HelpLabel.vue";
import { translate } from "./i18n";
import {
  friendlyTemplate,
  storedTemplate,
  templateValues,
} from "./template-values";
import { insertTemplateValue, type Locale } from "./ui";

const props = defineProps<{
  modelValue: string;
  locale: Locale;
  label: string;
  help: string;
  disabled?: boolean;
}>();
const emit = defineEmits<{ "update:modelValue": [value: string] }>();
const id = useId();
const input = ref<HTMLInputElement>();
const t = (key: Parameters<typeof translate>[1]) =>
  translate(props.locale, key);
const text = computed({
  get: () => friendlyTemplate(props.modelValue, t),
  set: (value) => emit("update:modelValue", storedTemplate(value, t)),
});
async function insert(event: Event) {
  const select = event.target as HTMLSelectElement;
  if (!select.value) return;
  const result = insertTemplateValue(
    text.value,
    input.value?.selectionStart ?? text.value.length,
    input.value?.selectionEnd ?? text.value.length,
    `[${select.value}]`,
  );
  text.value = result.value;
  select.value = "";
  await nextTick();
  input.value?.focus();
  input.value?.setSelectionRange(result.cursor, result.cursor);
}
</script>

<template>
  <div class="field">
    <HelpLabel :for-id="id" :label="label" :help="help" />
    <input
      :id="id"
      ref="input"
      v-model="text"
      :disabled="disabled"
      spellcheck="false"
      autocomplete="off"
    />
    <div class="insert-row">
      <HelpLabel
        :for-id="`${id}-value`"
        :label="t('presence.addValue')"
        :help="t('presence.addValueHelp')"
      />
      <select :id="`${id}-value`" :disabled="disabled" @change="insert">
        <option value="">{{ t("common.choose") }}</option>
        <option
          v-for="value in templateValues"
          :key="value.key"
          :value="t(value.key)"
        >
          {{ t(value.key) }}
        </option>
      </select>
    </div>
  </div>
</template>
