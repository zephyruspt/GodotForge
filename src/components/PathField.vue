<script setup lang="ts">
withDefaults(
  defineProps<{
    modelValue: string;
    buttonLabel: string;
    placeholder?: string;
    required?: boolean;
    disabled?: boolean;
    inputClass?: string;
  }>(),
  {
    placeholder: "",
    required: false,
    disabled: false,
    inputClass: "",
  },
);

const emit = defineEmits<{
  "update:modelValue": [value: string];
  browse: [];
}>();

function handleInput(event: Event) {
  emit("update:modelValue", (event.target as HTMLInputElement).value);
}
</script>

<template>
  <div class="flex gap-2">
    <input
      :value="modelValue"
      class="input input-bordered min-w-0 flex-1 border-base-content/10 bg-base-200"
      :class="inputClass"
      :placeholder="placeholder"
      :required="required"
      :disabled="disabled"
      @input="handleInput"
    />
    <button class="btn border-base-content/10 bg-base-content/5" type="button" :disabled="disabled" @click="emit('browse')">
      {{ buttonLabel }}
    </button>
  </div>
</template>
