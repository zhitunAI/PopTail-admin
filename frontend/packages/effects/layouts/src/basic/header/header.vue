<script lang="ts" setup>
import { computed, useSlots } from 'vue';

import { RotateCw } from '@vben/icons';
import { preferences, usePreferences } from '@vben/preferences';
import { useAccessStore } from '@vben/stores';

import { VbenFullScreen, VbenIconButton } from '@vben-core/shadcn-ui';

import {
  GlobalSearch,
  LanguageToggle,
  PreferencesButton,
  ThemeToggle,
  TimezoneButton,
} from '../../widgets';

interface Props {
  /**
   * Logo 主题
   */
  theme?: string;
}

defineOptions({
  name: 'LayoutHeader',
});

withDefaults(defineProps<Props>(), {
  theme: 'light',
});

const emit = defineEmits<{ clearPreferencesAndLogout: [] }>();

const REFERENCE_VALUE = 50;
const RELOAD_LOADING_ID = '__gaa-reload-loading__';
const RELOAD_LOADING_STYLE_ID = '__gaa-reload-loading-style__';

const accessStore = useAccessStore();
const { globalSearchShortcutKey, preferencesButtonPosition } = usePreferences();
const slots = useSlots();
const isDesktopShell =
  typeof window !== 'undefined' && !!(window as any).__TAURI_INTERNALS__?.invoke;

const rightSlots = computed(() => {
  const list = [{ index: REFERENCE_VALUE + 100, name: 'user-dropdown' }];
  if (preferences.widget.globalSearch) {
    list.push({
      index: REFERENCE_VALUE,
      name: 'global-search',
    });
  }

  if (preferencesButtonPosition.value.header) {
    list.push({
      index: REFERENCE_VALUE + 10,
      name: 'preferences',
    });
  }
  if (preferences.widget.themeToggle) {
    list.push({
      index: REFERENCE_VALUE + 20,
      name: 'theme-toggle',
    });
  }
  if (preferences.widget.languageToggle) {
    list.push({
      index: REFERENCE_VALUE + 30,
      name: 'language-toggle',
    });
  }
  if (preferences.widget.timezone) {
    list.push({
      index: REFERENCE_VALUE + 40,
      name: 'timezone',
    });
  }
  if (preferences.widget.fullscreen && !isDesktopShell) {
    list.push({
      index: REFERENCE_VALUE + 50,
      name: 'fullscreen',
    });
  }
  if (preferences.widget.notification) {
    list.push({
      index: REFERENCE_VALUE + 60,
      name: 'notification',
    });
  }

  Object.keys(slots).forEach((key) => {
    const name = key.split('-');
    if (key.startsWith('header-right')) {
      list.push({ index: Number(name[2]), name: key });
    }
  });
  return list.toSorted((a, b) => a.index - b.index);
});

const leftSlots = computed(() => {
  const list: Array<{ index: number; name: string }> = [];

  if (preferences.widget.refresh) {
    list.push({
      index: 0,
      name: 'refresh',
    });
  }

  Object.keys(slots).forEach((key) => {
    const name = key.split('-');
    if (key.startsWith('header-left')) {
      list.push({ index: Number(name[2]), name: key });
    }
  });
  return list.toSorted((a, b) => a.index - b.index);
});

function clearPreferencesAndLogout() {
  emit('clearPreferencesAndLogout');
}

function ensureReloadLoadingStyle() {
  if (document.getElementById(RELOAD_LOADING_STYLE_ID)) {
    return;
  }

  const style = document.createElement('style');
  style.id = RELOAD_LOADING_STYLE_ID;
  style.textContent = `
    #${RELOAD_LOADING_ID} {
      position: fixed;
      inset: 0;
      z-index: 99999;
      display: flex;
      align-items: center;
      justify-content: center;
      background: color-mix(in srgb, var(--background, #ffffff) 74%, transparent);
      backdrop-filter: blur(10px);
    }

    #${RELOAD_LOADING_ID} .gaa-reload-shell {
      display: flex;
      flex-direction: column;
      align-items: center;
      gap: 16px;
    }

    #${RELOAD_LOADING_ID} .gaa-reload-logo-wrap {
      position: relative;
      width: 88px;
      height: 88px;
      border-radius: 24px;
      display: flex;
      align-items: center;
      justify-content: center;
      background: color-mix(in srgb, var(--card, #ffffff) 88%, transparent);
      box-shadow:
        0 18px 50px color-mix(in srgb, var(--primary, #1677ff) 22%, transparent),
        inset 0 0 0 1px color-mix(in srgb, var(--primary, #1677ff) 16%, transparent);
      overflow: hidden;
    }

    #${RELOAD_LOADING_ID} .gaa-reload-logo-wrap::before,
    #${RELOAD_LOADING_ID} .gaa-reload-logo-wrap::after {
      content: '';
      position: absolute;
      inset: -20%;
      border-radius: 30px;
      border: 2px solid transparent;
      opacity: 0.55;
      animation: gaa-reload-ring 1.8s ease-in-out infinite;
    }

    #${RELOAD_LOADING_ID} .gaa-reload-logo-wrap::before {
      border-top-color: color-mix(in srgb, var(--primary, #1677ff) 90%, white);
      border-right-color: color-mix(in srgb, var(--primary, #1677ff) 45%, transparent);
    }

    #${RELOAD_LOADING_ID} .gaa-reload-logo-wrap::after {
      inset: -32%;
      animation-delay: 0.25s;
      border-bottom-color: color-mix(in srgb, var(--primary, #1677ff) 78%, white);
      border-left-color: color-mix(in srgb, var(--primary, #1677ff) 38%, transparent);
    }

    #${RELOAD_LOADING_ID} .gaa-reload-logo {
      width: 52px;
      height: 52px;
      object-fit: contain;
      animation: gaa-reload-logo 1.15s ease-in-out infinite alternate;
      filter: drop-shadow(0 10px 16px color-mix(in srgb, var(--primary, #1677ff) 30%, transparent));
    }

    #${RELOAD_LOADING_ID} .gaa-reload-text {
      font-size: 13px;
      color: color-mix(in srgb, var(--foreground, #111827) 72%, transparent);
      letter-spacing: 0.08em;
    }

    @keyframes gaa-reload-ring {
      0% {
        transform: rotate(0deg) scale(0.94);
        opacity: 0.2;
      }
      60% {
        opacity: 0.72;
      }
      100% {
        transform: rotate(360deg) scale(1.06);
        opacity: 0.18;
      }
    }

    @keyframes gaa-reload-logo {
      0% {
        transform: translateY(0) scale(0.96);
      }
      100% {
        transform: translateY(-2px) scale(1.04);
      }
    }
  `;
  document.head.appendChild(style);
}

function showReloadLoading() {
  if (typeof document === 'undefined') {
    return;
  }

  ensureReloadLoadingStyle();

  const existing = document.getElementById(RELOAD_LOADING_ID);
  if (existing) {
    return;
  }

  const overlay = document.createElement('div');
  overlay.id = RELOAD_LOADING_ID;
  const logo = preferences.logo.sourceDark || preferences.logo.source;
  overlay.innerHTML = `
    <div class="gaa-reload-shell" aria-live="polite" aria-label="loading">
      <div class="gaa-reload-logo-wrap">
        <img class="gaa-reload-logo" src="${logo}" alt="logo loading" />
      </div>
      <div class="gaa-reload-text">加载中...</div>
    </div>
  `;
  document.body.appendChild(overlay);
}

function reloadApp() {
  showReloadLoading();
  window.setTimeout(() => {
    window.location.reload();
  }, 60);
}
</script>

<template>
  <template
    v-for="slot in leftSlots.filter((item) => item.index < REFERENCE_VALUE)"
    :key="slot.name"
  >
    <slot :name="slot.name">
      <template v-if="slot.name === 'refresh'">
        <VbenIconButton class="my-0 mr-1 rounded-md" @click="reloadApp">
          <RotateCw class="size-4" />
        </VbenIconButton>
      </template>
    </slot>
  </template>
  <div class="flex min-w-0 items-center">
    <slot name="breadcrumb"></slot>
  </div>
  <template
    v-for="slot in leftSlots.filter((item) => item.index > REFERENCE_VALUE)"
    :key="slot.name"
  >
    <slot :name="slot.name"></slot>
  </template>
  <div
    :class="`menu-align-${preferences.header.menuAlign}`"
    class="flex h-full min-w-0 flex-1 items-center"
  >
    <slot name="menu"></slot>
  </div>
  <div class="flex h-full min-w-0 shrink-0 items-center">
    <template v-for="slot in rightSlots" :key="slot.name">
      <slot :name="slot.name">
        <template v-if="slot.name === 'global-search'">
          <GlobalSearch
            :enable-shortcut-key="globalSearchShortcutKey"
            :menus="accessStore.accessMenus"
            class="mr-1 sm:mr-4"
          />
        </template>

        <template v-else-if="slot.name === 'preferences'">
          <PreferencesButton
            class="mr-1"
            @clear-preferences-and-logout="clearPreferencesAndLogout"
          />
        </template>
        <template v-else-if="slot.name === 'theme-toggle'">
          <ThemeToggle class="mt-0.5 mr-1" />
        </template>
        <template v-else-if="slot.name === 'language-toggle'">
          <LanguageToggle class="mr-1" />
        </template>
        <template v-else-if="slot.name === 'fullscreen'">
          <VbenFullScreen class="mr-1" />
        </template>
        <template v-else-if="slot.name === 'timezone'">
          <TimezoneButton class="mt-0.5 mr-1" />
        </template>
      </slot>
    </template>
  </div>
</template>
<style lang="scss" scoped>
.menu-align-start {
  --menu-align: start;
}

.menu-align-center {
  --menu-align: center;
}

.menu-align-end {
  --menu-align: end;
}
</style>
