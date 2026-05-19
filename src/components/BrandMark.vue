<script setup lang="ts">
import { computed } from "vue";

const props = withDefaults(
  defineProps<{
    logo: string;
    size?: "sm" | "md";
  }>(),
  {
    size: "md",
  },
);

const wrapperClass = computed(() => (props.size === "sm" ? "h-9 w-9" : "h-10 w-10"));
const imageClass = computed(() => (props.size === "sm" ? "h-7 w-7" : "h-8 w-8"));
</script>

<template>
  <div
    class="grid shrink-0 place-items-center rounded-md bg-primary/10 shadow-lg shadow-primary/20 ring-1 ring-base-content/10"
    :class="wrapperClass"
  >
    <img :src="logo" alt="Godot Forge" class="object-contain" :class="imageClass" />
  </div>
</template>
