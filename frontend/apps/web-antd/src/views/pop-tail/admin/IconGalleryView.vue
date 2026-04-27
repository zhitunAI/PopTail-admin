<script setup lang="ts">

import { Page } from '@vben/common-ui';
import { IconifyIcon } from '@vben/icons';

import { message } from 'ant-design-vue';
import { usePageRefresh } from '#/utils/page-refresh';

const iconNames = [
  'lucide:activity-square',
  'lucide:alert-circle',
  'lucide:archive',
  'lucide:badge-info',
  'lucide:beaker',
  'lucide:book-a',
  'lucide:bot',
  'lucide:brain-circuit',
  'lucide:briefcase',
  'lucide:calendar-clock',
  'lucide:clipboard-list',
  'lucide:download',
  'lucide:file-code-2',
  'lucide:file-pen-line',
  'lucide:files',
  'lucide:flask-conical',
  'lucide:git-compare-arrows',
  'lucide:history',
  'lucide:image',
  'lucide:image-plus',
  'lucide:key',
  'lucide:key-round',
  'lucide:layers',
  'lucide:layout-dashboard',
  'lucide:library-big',
  'lucide:list-tree',
  'lucide:logs',
  'lucide:mail',
  'lucide:megaphone',
  'lucide:menu-square',
  'lucide:monitor-smartphone',
  'lucide:package-open',
  'lucide:package-plus',
  'lucide:pause-circle',
  'lucide:plug-zap',
  'lucide:refresh-cw',
  'lucide:rocket',
  'lucide:scan-line',
  'lucide:separator-horizontal',
  'lucide:server',
  'lucide:server-cog',
  'lucide:settings-2',
  'lucide:shield',
  'lucide:sliders-horizontal',
  'lucide:sparkles',
  'lucide:square-pen',
  'lucide:terminal',
  'lucide:triangle-alert',
  'lucide:upload-cloud',
  'lucide:user-circle-2',
  'lucide:users',
  'lucide:users-round',
  'lucide:wand',
  'lucide:waypoints',
  'lucide:wrench',
  'carbon:activity',
  'carbon:add',
  'carbon:apps',
  'carbon:archive',
  'carbon:arrow-right',
  'carbon:arrow-up-right',
  'carbon:bot',
  'carbon:category',
  'carbon:checkmark',
  'carbon:checkmark-outline',
  'carbon:close',
  'carbon:code',
  'carbon:compass',
  'carbon:data-base',
  'carbon:delivery',
  'carbon:document',
  'carbon:download',
  'carbon:edit',
  'carbon:flow-stream',
  'carbon:folder',
  'carbon:ibm-cloud-projects',
  'carbon:image',
  'carbon:information',
  'carbon:launch',
  'carbon:link',
  'carbon:list',
  'carbon:login',
  'carbon:menu',
  'carbon:monitoring',
  'carbon:renew',
  'carbon:rocket',
  'carbon:security',
  'carbon:server',
  'carbon:settings',
  'carbon:sparkle',
  'carbon:terminal',
  'carbon:timer',
  'carbon:tools',
  'carbon:trash-can',
  'carbon:user',
  'carbon:user-avatar',
  'carbon:user-multiple',
  'carbon:user-profile',
  'carbon:user-role',
  'carbon:warning-alt',
  'mdi:account-badge-outline',
  'mdi:account-key-outline',
  'mdi:alert-outline',
  'mdi:api',
  'mdi:archive-outline',
  'mdi:auto-fix',
  'mdi:calendar-clock-outline',
  'mdi:clipboard-alert-outline',
  'mdi:clock-outline',
  'mdi:comment-processing-outline',
  'mdi:database-outline',
  'mdi:file-eye-outline',
  'mdi:file-tree-outline',
  'mdi:form-select',
  'mdi:history',
  'mdi:image-outline',
  'mdi:package-variant-closed',
  'mdi:package-variant-closed-check',
  'mdi:robot-outline',
  'mdi:script-text-outline',
  'mdi:shape-outline',
  'mdi:shield-alert-outline',
  'mdi:shield-check-outline',
  'mdi:table-column-plus-after',
  'mdi:toolbox-outline',
  'mdi:view-dashboard-outline',
];

const icons = [...new Set(iconNames)].sort((left, right) => left.localeCompare(right));

async function copyIcon(icon: string) {
  await navigator.clipboard.writeText(icon);
  message.success(`已复制 ${icon}`);
}

usePageRefresh(async () => {});
</script>

<template>
  <Page auto-content-height title="图标库">
    <div class="relative flex h-full min-h-0 flex-1 flex-col">
      <div class="icon-grid">
      <button
        v-for="icon in icons"
        :key="icon"
        class="icon-item"
        type="button"
        @click="copyIcon(icon)"
      >
        <IconifyIcon :icon="icon" class="icon-preview" />
        <span>{{ icon }}</span>
      </button>
      </div>
    </div>
  </Page>
</template>

<style scoped>
.icon-grid {
  display: grid;
  gap: 12px;
  grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
}

.icon-item {
  align-items: center;
  background: var(--ant-color-bg-container);
  border: 1px solid var(--ant-color-border);
  border-radius: 8px;
  color: var(--ant-color-text);
  cursor: pointer;
  display: grid;
  gap: 8px;
  justify-items: center;
  min-height: 112px;
  padding: 14px 10px;
  transition: border-color 0.16s ease, color 0.16s ease;
}

.icon-item:hover {
  border-color: var(--ant-color-primary);
  color: var(--ant-color-primary);
}

.icon-preview {
  font-size: 30px;
}

.icon-item span {
  font-size: 12px;
  line-height: 1.4;
  max-width: 100%;
  overflow: hidden;
  text-align: center;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
