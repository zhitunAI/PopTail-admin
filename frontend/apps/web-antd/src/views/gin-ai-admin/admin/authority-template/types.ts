import type { AuthorityInfo } from '#/types/gin-ai-admin';

export interface RoleRow {
  authority: AuthorityInfo;
  createTime: string;
  defaultRouter: string;
  id: string;
  name: string;
  parentId: number;
  permissions: string[];
  remark: string;
  status: 0 | 1;
}
