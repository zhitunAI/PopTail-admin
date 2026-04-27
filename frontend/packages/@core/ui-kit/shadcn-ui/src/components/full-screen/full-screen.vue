<script lang="ts" setup>
import { Maximize, Minimize } from '@vben-core/icons';

import { onMounted, ref } from 'vue';
import { useFullscreen } from '@vueuse/core';

import { VbenIconButton } from '../button';

defineOptions({ name: 'FullScreen' });

const { isFullscreen, toggle } = useFullscreen();
const desktopFullscreen = ref(false);

type DesktopFullscreenCache = {
  initialized: boolean;
  pending?: Promise<boolean>;
  value: boolean;
};

function tauriInvoke(command: string) {
  return (window as any).__TAURI_INTERNALS__?.invoke?.(command);
}

function isDesktopShell() {
  return typeof window !== 'undefined' && !!(window as any).__TAURI_INTERNALS__?.invoke;
}

function getDesktopFullscreenCache(): DesktopFullscreenCache | null {
  if (typeof window === 'undefined') {
    return null;
  }
  const cacheKey = '__POP_TAIL_DESKTOP_FULLSCREEN_CACHE__';
  const existing = (window as any)[cacheKey] as DesktopFullscreenCache | undefined;
  if (existing) {
    return existing;
  }
  const created: DesktopFullscreenCache = {
    initialized: false,
    value: false,
  };
  (window as any)[cacheKey] = created;
  return created;
}

function setDesktopFullscreen(value: boolean) {
  desktopFullscreen.value = value;
  const cache = getDesktopFullscreenCache();
  if (cache) {
    cache.initialized = true;
    cache.value = value;
  }
}

async function resolveDesktopFullscreen() {
  const cache = getDesktopFullscreenCache();
  if (!cache) {
    return false;
  }
  if (cache.initialized) {
    return cache.value;
  }
  if (!cache.pending) {
    cache.pending = tauriInvoke('desktop_is_fullscreen')
      .then((value: boolean) => {
        cache.initialized = true;
        cache.value = Boolean(value);
        return cache.value;
      })
      .catch(() => {
        cache.initialized = true;
        cache.value = false;
        return false;
      })
      .finally(() => {
        cache.pending = undefined;
      });
  }
  return await cache.pending;
}

async function toggleFullscreen() {
  if (isDesktopShell()) {
    try {
      setDesktopFullscreen(await tauriInvoke('desktop_toggle_fullscreen'));
      return;
    } catch {
      // fall through to browser fullscreen when desktop bridge is unavailable
    }
  }
  await toggle();
}

// 重新检查全屏状态
isFullscreen.value = !!(
  document.fullscreenElement ||
  // @ts-expect-error - vendor fullscreen APIs are not included in the standard DOM typings
  document.webkitFullscreenElement ||
  // @ts-expect-error - vendor fullscreen APIs are not included in the standard DOM typings
  document.mozFullScreenElement ||
  // @ts-expect-error - vendor fullscreen APIs are not included in the standard DOM typings
  document.msFullscreenElement
);

onMounted(async () => {
  if (!isDesktopShell()) {
    return;
  }
  setDesktopFullscreen(Boolean(await resolveDesktopFullscreen()));
});
</script>
<template>
  <VbenIconButton
    class="hover:animate-[shrink_0.3s_ease-in-out]"
    @click="toggleFullscreen"
  >
    <Minimize v-if="desktopFullscreen || isFullscreen" class="text-foreground size-4" />
    <Maximize v-else class="text-foreground size-4" />
  </VbenIconButton>
</template>
