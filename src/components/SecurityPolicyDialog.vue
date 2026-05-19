<script setup lang="ts">
import { useI18n } from "vue-i18n";

defineProps<{
  open: boolean;
}>();

const emit = defineEmits<{
  close: [];
}>();

const { t } = useI18n();
</script>

<template>
  <dialog class="modal" :open="open">
    <div class="modal-box border border-base-content/10 bg-base-100">
      <h3 class="text-xl font-black">{{ t("security.title") }}</h3>
      <div class="mt-4 grid gap-3 text-sm text-base-content/75">
        <p>{{ t("security.ruleCredentials") }}</p>
        <p>{{ t("security.ruleLocal") }}</p>
        <p>{{ t("security.ruleReporting") }}</p>
      </div>
      <div class="modal-action">
        <button class="btn btn-primary" type="button" @click="emit('close')">
          {{ t("common.close") }}
        </button>
      </div>
    </div>
    <form class="modal-backdrop" method="dialog" @submit.prevent="emit('close')">
      <button>{{ t("common.close") }}</button>
    </form>
  </dialog>
</template>
