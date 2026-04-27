import { onMounted, onUnmounted } from 'vue';
import { useRoute } from 'vue-router';

export const PAGE_REFRESH_EVENT = 'gaa:page-refresh';

type PageRefreshDetail = {
  name?: null | string;
  path?: string;
};

export function dispatchPageRefresh(detail: PageRefreshDetail) {
  if (typeof window === 'undefined') {
    return false;
  }
  const event = new CustomEvent<PageRefreshDetail>(PAGE_REFRESH_EVENT, {
    cancelable: true,
    detail,
  });
  return !window.dispatchEvent(event);
}

export function usePageRefresh(handler: () => void | Promise<void>) {
  const route = useRoute();

  const listener = (event: Event) => {
    const customEvent = event as CustomEvent<PageRefreshDetail>;
    const detail = customEvent.detail ?? {};
    const routeName = typeof route.name === 'string' ? route.name : '';
    const targetName = detail.name ?? '';
    const targetPath = detail.path ?? '';

    const matched =
      (targetName && routeName && targetName === routeName) ||
      (targetPath && targetPath === route.path);

    if (!matched) {
      return;
    }

    customEvent.preventDefault();
    void handler();
  };

  onMounted(() => {
    window.addEventListener(PAGE_REFRESH_EVENT, listener as EventListener);
  });

  onUnmounted(() => {
    window.removeEventListener(PAGE_REFRESH_EVENT, listener as EventListener);
  });
}
