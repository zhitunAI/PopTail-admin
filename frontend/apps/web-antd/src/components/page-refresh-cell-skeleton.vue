<script lang="ts" setup>
import { Skeleton } from 'ant-design-vue';

defineOptions({ name: 'PageRefreshCellSkeleton' });

interface Props {
  width?: string;
}

withDefaults(defineProps<Props>(), {
  width: '70%',
});
</script>

<template>
  <div class="page-refresh-cell-skeleton">
    <Skeleton active :paragraph="false" :title="{ width }" />
  </div>
</template>

<style scoped>
.page-refresh-cell-skeleton {
  display: flex;
  min-height: 32px;
  align-items: center;
  width: 100%;
}

.page-refresh-cell-skeleton :deep(.ant-skeleton) {
  width: 100%;
}

.page-refresh-cell-skeleton :deep(.ant-skeleton-title) {
  margin-block: 0;
}
</style>
