import type { MenuInfo } from '#/types/pop-tail';

export type TemplateMenuType = 'button' | 'catalog' | 'embedded' | 'link' | 'menu';

export interface MenuRow {
  authCode: string;
  children?: MenuRow[];
  component: string;
  id: string;
  level: number;
  loading?: boolean;
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
  sort: number;
  status: '隐藏' | '显示';
  type: TemplateMenuType;
}
