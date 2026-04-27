<script lang="ts" setup>
import { Skeleton } from 'ant-design-vue';

defineOptions({ name: 'PageRefreshSkeleton' });

interface Props {
  rows?: number;
  type?: 'cards' | 'detail' | 'form' | 'list';
}

const props = withDefaults(defineProps<Props>(), {
  rows: 6,
  type: 'list',
});
</script>

<template>
  <div class="page-refresh-skeleton">
    <template v-if="props.type === 'cards'">
      <div class="page-refresh-skeleton__cards">
        <div v-for="item in 4" :key="item" class="page-refresh-skeleton__card">
          <Skeleton active :paragraph="{ rows: 2 }" :title="{ width: '42%' }" />
        </div>
      </div>
    </template>

    <template v-else-if="props.type === 'form'">
      <div class="page-refresh-skeleton__form">
        <div v-for="item in props.rows" :key="item" class="page-refresh-skeleton__field">
          <Skeleton active :paragraph="{ rows: 1, width: ['100%'] }" :title="{ width: '24%' }" />
        </div>
      </div>
    </template>

    <template v-else-if="props.type === 'detail'">
      <div class="page-refresh-skeleton__detail">
        <div v-for="item in props.rows" :key="item" class="page-refresh-skeleton__detail-row">
          <Skeleton active :paragraph="{ rows: 1, width: [`${68 + (item % 3) * 8}%`] }" :title="{ width: '18%' }" />
        </div>
      </div>
    </template>

    <template v-else>
      <div class="page-refresh-skeleton__list">
        <div v-for="item in props.rows" :key="item" class="page-refresh-skeleton__list-row">
          <Skeleton active :paragraph="{ rows: 1, width: [`${76 + (item % 3) * 6}%`] }" :title="{ width: `${38 + (item % 4) * 10}%` }" />
        </div>
      </div>
    </template>
  </div>
</template>

<style scoped>
.page-refresh-skeleton {
  width: 100%;
}

.page-refresh-skeleton__cards {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 16px;
  padding: 8px 4px;
}

.page-refresh-skeleton__card,
.page-refresh-skeleton__field,
.page-refresh-skeleton__detail-row,
.page-refresh-skeleton__list-row {
  padding: 8px 10px;
}

.page-refresh-skeleton__form,
.page-refresh-skeleton__detail,
.page-refresh-skeleton__list {
  display: grid;
  gap: 12px;
  padding: 8px 4px;
}

.page-refresh-skeleton :deep(.ant-skeleton) {
  width: 100%;
  pointer-events: none;
}

.page-refresh-skeleton :deep(.ant-skeleton-title),
.page-refresh-skeleton :deep(.ant-skeleton-paragraph) {
  margin-block-start: 0;
  margin-block-end: 0;
}
</style>
