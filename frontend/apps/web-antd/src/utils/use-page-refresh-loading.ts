import { ref } from 'vue';

import { usePageRefresh } from './page-refresh';

export function usePageRefreshLoading(refresher: () => Promise<void>) {
  const pageLoading = ref(false);

  async function onPageRefresh() {
    if (pageLoading.value) {
      return;
    }
    pageLoading.value = true;
    try {
      await refresher();
    } finally {
      pageLoading.value = false;
    }
  }

  usePageRefresh(onPageRefresh);

  return {
    onPageRefresh,
    pageLoading,
  };
}
