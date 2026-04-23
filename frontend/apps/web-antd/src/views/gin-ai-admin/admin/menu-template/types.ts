import type { MenuInfo } from '#/types/gin-ai-admin';

export type TemplateMenuType = 'button' | 'catalog' | 'embedded' | 'link' | 'menu';

export interface MenuRow {
  authCode: string;
  children?: MenuRow[];
  component: string;
  id: string;
  menu: MenuInfo;
  meta: {
    badge?: string;
    badgeType?: 'dot' | 'normal';
    badgeVariants?: 'default' | 'destructive' | 'primary' | 'success' | 'warning';
    icon?: string;
    iframeSrc?: string;
    link?: string;
    title: string;
  };
  name: string;
  path: string;
  pid: string;
  status: '隐藏' | '显示';
  type: TemplateMenuType;
}
