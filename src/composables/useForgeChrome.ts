import { computed, ref, watch, type Ref, type WritableComputedRef } from "vue";
import { getVersion } from "@tauri-apps/api/app";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { Section, ThemeName } from "../types";
import type { Translator } from "../utils/forgeHelpers";

const localeStorageKey = "godot-forge-locale";
const themeStorageKey = "godot-forge-theme";
const onboardingStorageKey = "godot-forge-onboarding-complete";
const themeNames: ThemeName[] = ["godotforge", "godotforge-light"];

type UseForgeChromeOptions = {
  locale: WritableComputedRef<string>;
  busyAction: Ref<string>;
  error: Ref<string>;
  status: Ref<string>;
  t: Translator;
  navigateSection: (section: Section) => void;
};

export function useForgeChrome({ locale, busyAction, error, status, t, navigateSection }: UseForgeChromeOptions) {
  const savedLocale = localStorage.getItem(localeStorageKey);
  const savedTheme = localStorage.getItem(themeStorageKey);

  if (savedLocale === "en" || savedLocale === "pt") {
    locale.value = savedLocale;
  }

  const appVersion = ref("");
  const selectedLocale = computed({
    get: () => locale.value,
    set: (value: string) => {
      locale.value = value;
      localStorage.setItem(localeStorageKey, value);
    },
  });
  const selectedTheme = ref<ThemeName>(
    themeNames.includes(savedTheme as ThemeName) ? (savedTheme as ThemeName) : "godotforge",
  );
  const showWelcome = ref(localStorage.getItem(onboardingStorageKey) !== "true");
  const welcomeSlide = ref(0);
  const securityDialogOpen = ref(false);
  let unlistenMenuAction: UnlistenFn | null = null;
  let toastTimer: ReturnType<typeof setTimeout> | null = null;

  const welcomeSlides = computed(() => [
    { label: t("onboarding.slideOneLabel"), title: t("onboarding.slideOneTitle"), body: t("onboarding.slideOneBody") },
    { label: t("onboarding.slideTwoLabel"), title: t("onboarding.slideTwoTitle"), body: t("onboarding.slideTwoBody") },
    { label: t("onboarding.slideThreeLabel"), title: t("onboarding.slideThreeTitle"), body: t("onboarding.slideThreeBody") },
  ]);

  async function loadAppMetadata() {
    try {
      appVersion.value = await getVersion();
    } catch {
      appVersion.value = "";
    }
  }

  function handleMenuAction(action: string) {
    if (action === "dashboard" || action === "projects" || action === "editors" || action === "diagnostics" || action === "settings") {
      navigateSection(action);
      return;
    }
    if (action === "security-policy") securityDialogOpen.value = true;
  }

  async function listenForMenuActions() {
    try {
      unlistenMenuAction = await listen<string>("menu-action", (event) => handleMenuAction(event.payload));
    } catch {
      unlistenMenuAction = null;
    }
  }

  function completeWelcome(section: Section = "projects") {
    showWelcome.value = false;
    localStorage.setItem(onboardingStorageKey, "true");
    navigateSection(section);
  }

  function nextWelcomeSlide() {
    if (welcomeSlide.value >= welcomeSlides.value.length - 1) {
      completeWelcome("projects");
      return;
    }
    welcomeSlide.value += 1;
  }

  function previousWelcomeSlide() {
    welcomeSlide.value = Math.max(0, welcomeSlide.value - 1);
  }

  function clearToastTimer() {
    if (!toastTimer) return;
    clearTimeout(toastTimer);
    toastTimer = null;
  }

  function disposeChrome() {
    clearToastTimer();
    unlistenMenuAction?.();
  }

  watch(selectedTheme, (theme) => {
    document.documentElement.dataset.theme = theme;
    localStorage.setItem(themeStorageKey, theme);
  });

  watch([status, error, busyAction], ([nextStatus, nextError, nextBusyAction]) => {
    clearToastTimer();
    if (nextBusyAction || (!nextStatus && !nextError)) return;
    const currentStatus = nextStatus;
    const currentError = nextError;
    toastTimer = setTimeout(
      () => {
        if (currentStatus && status.value === currentStatus) status.value = "";
        if (currentError && error.value === currentError) error.value = "";
      },
      nextError ? 8000 : 3500,
    );
  });

  return {
    appVersion,
    selectedLocale,
    selectedTheme,
    showWelcome,
    welcomeSlide,
    securityDialogOpen,
    welcomeSlides,
    loadAppMetadata,
    listenForMenuActions,
    completeWelcome,
    nextWelcomeSlide,
    previousWelcomeSlide,
    disposeChrome,
  };
}
