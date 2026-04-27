import type { AuthorityInfo } from '#/types/pop-tail';

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
