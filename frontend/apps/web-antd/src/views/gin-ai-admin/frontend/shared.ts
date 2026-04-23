export type FrontendNavItem = {
  icon: string;
  id: number;
  order: number;
  path: string;
  title: string;
  visible: boolean;
};

export type ArticleCategoryItem = {
  id: number;
  name: string;
  slug: string;
  sort: number;
  status: boolean;
};

export type ArticleItem = {
  categoryId: number;
  content: string;
  id: number;
  slug: string;
  status: 'draft' | 'published';
  title: string;
  updatedAt: string;
};

export type MemberItem = {
  consolePath: string;
  email: string;
  id: number;
  lastLogin: string;
  nickname: string;
  provider: 'email' | 'google';
  status: 'active' | 'disabled';
};

export type ConsoleMenuItem = {
  group: string;
  icon: string;
  id: number;
  order: number;
  path: string;
  title: string;
  visible: boolean;
};

const STORAGE_KEYS = {
  articleCategories: 'gaa-frontend-article-categories',
  articles: 'gaa-frontend-articles',
  consoleMenus: 'gaa-frontend-console-menus',
  frontendNav: 'gaa-frontend-nav',
  members: 'gaa-frontend-members',
};

function readStore<T>(key: string, fallback: T[]): T[] {
  if (typeof window === 'undefined') return fallback;
  const raw = window.localStorage.getItem(key);
  if (!raw) return fallback;
  try {
    const parsed = JSON.parse(raw);
    return Array.isArray(parsed) ? parsed : fallback;
  } catch {
    return fallback;
  }
}

function writeStore<T>(key: string, value: T[]) {
  if (typeof window === 'undefined') return;
  window.localStorage.setItem(key, JSON.stringify(value));
}

export function loadFrontendNav() {
  return readStore<FrontendNavItem>(STORAGE_KEYS.frontendNav, [
    { id: 1, icon: 'lucide:house', order: 1, path: '/', title: '首页', visible: true },
    { id: 2, icon: 'lucide:newspaper', order: 2, path: '/articles', title: '文章', visible: true },
    { id: 3, icon: 'lucide:circle-user-round', order: 3, path: '/member', title: '会员中心', visible: true },
  ]);
}

export function saveFrontendNav(items: FrontendNavItem[]) {
  writeStore(STORAGE_KEYS.frontendNav, items);
}

export function loadArticleCategories() {
  return readStore<ArticleCategoryItem>(STORAGE_KEYS.articleCategories, [
    { id: 1, name: '产品公告', slug: 'product-news', sort: 1, status: true },
    { id: 2, name: '帮助文档', slug: 'docs', sort: 2, status: true },
  ]);
}

export function saveArticleCategories(items: ArticleCategoryItem[]) {
  writeStore(STORAGE_KEYS.articleCategories, items);
}

export function loadArticles() {
  return readStore<ArticleItem>(STORAGE_KEYS.articles, [
    {
      id: 1,
      categoryId: 1,
      content: '# 欢迎使用\n\n这里是文章内容编辑区。',
      slug: 'welcome',
      status: 'published',
      title: '欢迎使用',
      updatedAt: '2026-04-22 09:00:00',
    },
  ]);
}

export function saveArticles(items: ArticleItem[]) {
  writeStore(STORAGE_KEYS.articles, items);
}

export function loadMembers() {
  return readStore<MemberItem>(STORAGE_KEYS.members, [
    {
      id: 1,
      consolePath: '/member/console',
      email: 'member@example.com',
      lastLogin: '2026-04-22 08:30:00',
      nickname: '示例会员',
      provider: 'email',
      status: 'active',
    },
  ]);
}

export function saveMembers(items: MemberItem[]) {
  writeStore(STORAGE_KEYS.members, items);
}

export function loadConsoleMenus() {
  return readStore<ConsoleMenuItem>(STORAGE_KEYS.consoleMenus, [
    { id: 1, group: '账号', icon: 'lucide:user-round-cog', order: 1, path: '/member/profile', title: '资料修改', visible: true },
    { id: 2, group: '账号', icon: 'lucide:key-round', order: 2, path: '/member/security', title: '安全设置', visible: true },
  ]);
}

export function saveConsoleMenus(items: ConsoleMenuItem[]) {
  writeStore(STORAGE_KEYS.consoleMenus, items);
}

export function nextLocalId(items: Array<{ id: number }>) {
  return items.length > 0 ? Math.max(...items.map((item) => item.id)) + 1 : 1;
}
