<script setup lang="ts">
import { useI18n } from "vue-i18n";
import type { DeleteTarget } from "../types";

const props = defineProps<{
  open: boolean;
  deleteType: DeleteTarget;
  name: string;
  busy?: boolean;
}>();

const emit = defineEmits<{
  cancel: [];
  confirm: [];
}>();

const { t } = useI18n();
</script>

<template>
  <dialog class="modal" :open="open">
    <div class="modal-box border border-base-content/10 bg-base-100">
      <h3 class="text-xl font-black">{{ t("confirmDelete.title") }}</h3>
      <p class="mt-3 text-sm text-base-content/70">
        {{
          props.deleteType === "project"
            ? t("confirmDelete.projectBody", { name: props.name })
            : t("confirmDelete.editorBody", { name: props.name })
        }}
      </p>
      <div class="modal-action">
        <button class="btn border-base-content/10 bg-base-content/5" type="button" @click="emit('cancel')">
          {{ t("common.cancel") }}
        </button>
        <button class="btn btn-error" type="button" :disabled="busy" @click="emit('confirm')">
          {{ t("common.delete") }}
        </button>
      </div>
    </div>
    <form class="modal-backdrop" method="dialog" @submit.prevent="emit('cancel')">
      <button>{{ t("common.cancel") }}</button>
    </form>
  </dialog>
</template>
