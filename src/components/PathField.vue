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
  <div class="join w-full">
    <label class="input join-item min-w-0 flex-1 border-base-content/10 bg-base-200">
      <svg
        class="h-[1em] shrink-0 opacity-50"
        xmlns="http://www.w3.org/2000/svg"
        viewBox="0 0 24 24"
        aria-hidden="true"
      >
        <g stroke-linejoin="round" stroke-linecap="round" stroke-width="2.25" fill="none" stroke="currentColor">
          <path d="M3 7.5A2.5 2.5 0 0 1 5.5 5h4.1c.7 0 1.36.33 1.78.89L12.2 7H18.5A2.5 2.5 0 0 1 21 9.5v7A2.5 2.5 0 0 1 18.5 19h-13A2.5 2.5 0 0 1 3 16.5z" />
          <path d="M3 10h18" />
        </g>
      </svg>
      <input
        :value="modelValue"
        class="min-w-0"
        :class="inputClass"
        :placeholder="placeholder"
        :required="required"
        :disabled="disabled"
        spellcheck="false"
        @input="handleInput"
      />
    </label>
    <button
      class="btn join-item w-28 shrink-0 cursor-pointer border-base-content/10 bg-base-content/5 px-3 hover:bg-base-content/10 disabled:cursor-not-allowed"
      type="button"
      :disabled="disabled"
      @click="emit('browse')"
    >
      {{ buttonLabel }}
    </button>
  </div>
</template>
