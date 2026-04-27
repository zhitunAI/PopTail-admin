import {
  defineOverridesPreferences,
  definePreferencesExtension,
} from '@vben/preferences';

interface WebAntdPreferencesExtension {
  defaultTableSize: number;
  enableFormFullscreen: boolean;
  reportTitle: string;
  tenantMode: 'multi' | 'single';
}

const DEFAULT_LOGO_URL = '/brand/logo.svg';
const SYSTEM_CONFIG_STORAGE_KEY = 'pop-tail-system-config-v1';

function resolveInitialLogoUrl() {
  if (typeof window === 'undefined') {
    return DEFAULT_LOGO_URL;
  }
  try {
    const raw = window.localStorage.getItem(SYSTEM_CONFIG_STORAGE_KEY);
    if (!raw) {
      return DEFAULT_LOGO_URL;
    }
    const parsed = JSON.parse(raw) as { system?: { logoUrl?: string } };
    const logoUrl = parsed.system?.logoUrl?.trim();
    return logoUrl || DEFAULT_LOGO_URL;
  } catch {
    return DEFAULT_LOGO_URL;
  }
}

const initialLogoUrl = resolveInitialLogoUrl();

export const overridesPreferences = defineOverridesPreferences({
  app: {
    accessMode: 'frontend',
    authPageLayout: 'panel-center',
    defaultHomePath: '/dashboard',
    loginExpiredMode: 'modal',
    name: import.meta.env.VITE_APP_TITLE,
  },
  logo: {
    source: initialLogoUrl,
    sourceDark: initialLogoUrl,
  },
  transition: {
    loading: false,
    progress: false,
  },
});

export const preferencesExtension =
  definePreferencesExtension<WebAntdPreferencesExtension>({
    tabLabel: 'preferences.antd.tabLabel',
    title: 'preferences.antd.title',
    fields: [
      {
        component: 'switch',
        defaultValue: true,
        key: 'enableFormFullscreen',
        label: 'preferences.antd.fields.enableFormFullscreen.label',
        tip: 'preferences.antd.fields.enableFormFullscreen.tip',
      },
      {
        component: 'select',
        defaultValue: 'single',
        key: 'tenantMode',
        label: 'preferences.antd.fields.tenantMode.label',
        options: [
          {
            label: 'preferences.antd.fields.tenantMode.options.single.label',
            value: 'single',
          },
          {
            label: 'preferences.antd.fields.tenantMode.options.multi.label',
            value: 'multi',
          },
        ],
      },
      {
        component: 'number',
        componentProps: {
          max: 200,
          min: 10,
          step: 10,
        },
        defaultValue: 20,
        key: 'defaultTableSize',
        label: 'preferences.antd.fields.defaultTableSize.label',
      },
      {
        component: 'input',
        defaultValue: '',
        key: 'reportTitle',
        label: 'preferences.antd.fields.reportTitle.label',
        placeholder: 'preferences.antd.fields.reportTitle.placeholder',
      },
    ],
  });
