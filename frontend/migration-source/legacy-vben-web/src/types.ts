export interface AuthorityInfo {
  ID: number;
  authorityId: number;
  authorityName: string;
  defaultRouter: string;
  parentId: number;
  children: AuthorityInfo[];
}

export interface UserInfo {
  ID: number;
  uuid: string;
  userName: string;
  nickName: string;
  authorityId: number;
  authority: AuthorityInfo;
  authorities: AuthorityInfo[];
  headerImg: string;
  phone: string;
  email: string;
  enable: number;
}

export interface ApiResponse<T> {
  code: number;
  msg: string;
  data: T;
}

export interface LoginResult {
  user: UserInfo;
  token: string;
  expiresAt: number;
  refreshToken: string;
}

export interface UserInfoResult {
  userInfo: UserInfo;
}

export interface PolicyPath {
  path: string;
  method: string;
}

export interface PageResult<T> {
  List: T[];
  Total: number;
  Page: number;
  PageSize: number;
}

export interface MenuMeta {
  title: string;
  icon: string;
}

export interface MenuInfo {
  ID: number;
  parentId: number;
  name: string;
  path: string;
  component: string;
  sort: number;
  hidden: boolean;
  meta: MenuMeta;
  children: MenuInfo[];
}

export interface MenuTreeResult {
  menus: MenuInfo[];
}

export interface ApiInfo {
  ID: number;
  path: string;
  apiGroup: string;
  description: string;
  method: string;
}

export interface DictionaryInfo {
  ID: number;
  name: string;
  type: string;
  status: boolean;
  desc: string;
}

export interface ParamInfo {
  ID: number;
  key: string;
  value: string;
  desc: string;
}

export interface ProfileUpdateInput {
  nickName: string;
  phone: string;
  email: string;
}

export interface UserUpsertInput {
  ID?: number;
  userName: string;
  nickName: string;
  authorityId: number;
  phone: string;
  email: string;
  enable: number;
}

export interface AuthorityUpsertInput {
  ID?: number;
  authorityId: number;
  authorityName: string;
  defaultRouter: string;
  parentId: number;
}

export interface DictionaryDetailUpsertInput {
  ID?: number;
  sysDictionaryID: number;
  label: string;
  value: string;
  extend: string;
  level: number;
  status: boolean;
  sort: number;
  parentID?: number | null;
}

export interface MenuUpsertInput {
  ID?: number;
  parentId: number;
  name: string;
  path: string;
  component: string;
  sort: number;
  hidden: boolean;
  meta: MenuMeta;
}

export interface ApiUpsertInput {
  ID?: number;
  path: string;
  apiGroup: string;
  description: string;
  method: string;
}

export interface DictionaryUpsertInput {
  ID?: number;
  name: string;
  type: string;
  status: boolean;
  desc: string;
}

export interface ApiTokenRecord {
  ID: number;
  name: string;
  scope: string;
  ttl: string;
  status: string;
}

export interface PackageRecord {
  ID: number;
  name: string;
  kind: string;
  output: string;
  summary: string;
}

export interface PluginManifestRecord {
  ID: number;
  pluginName: string;
  menuGroup: string;
  menuIds: number[];
  apiIds: number[];
  dictionaryIds: number[];
  savedAt: number;
}

export interface PluginInstallRecord {
  ID: number;
  name: string;
  kind: string;
  target: string;
  manifest: string;
  status: string;
  createdAt: number;
}

export interface AutoCodeRegistryRecord {
  ID: number;
  createdAt: number;
  payload: {
    entity: string;
    table: string;
    module: string;
    outputs: string[];
    [key: string]: unknown;
  };
}

export interface ReleaseRecord {
  ID: number;
  createdAt: number;
  note: string;
}

export interface EmailRecord {
  ID: number;
  to: string;
  subject: string;
  body: string;
  mode: string;
  status: string;
  createdAt: number;
}

export interface AnnouncementAttachment {
  name: string;
  url: string;
}

export interface AnnouncementRecord {
  ID: number;
  CreatedAt: number;
  UpdatedAt: number;
  title: string;
  content: string;
  userID: number;
  attachments: AnnouncementAttachment[];
}

export interface AnnouncementDataSource {
  userID: Array<{
    label: string;
    value: number;
  }>;
}

export interface UploadFileRecord {
  ID: number;
  name: string;
  size: string;
  status: string;
}

export interface UploadQueueMutationInput {
  files?: Array<{
    name: string;
    size: string;
  }>;
  mode?: "replace" | "append" | "inspect";
}

export interface ResumeUploadRecord {
  ID: number;
  name: string;
  progress: number;
  status: string;
}

export interface ScanSessionRecord {
  sessionId: string;
  status: string;
}

export interface CustomerRecord {
  ID: number;
  CreatedAt: number;
  UpdatedAt: number;
  customerName: string;
  customerPhoneData: string;
  sysUserId: number;
  customerLevel: string;
  customerStatus: string;
  remark: string;
}

export interface McpToolParam {
  name: string;
  description: string;
  type: string;
  required: boolean;
  defaultValue?: string | null;
}

export interface McpToolOutput {
  type: string;
}

export interface McpToolRecord {
  ID: number;
  createdAt: number;
  updatedAt: number;
  name: string;
  description: string;
  params: McpToolParam[];
  response: McpToolOutput[];
}

export interface McpSchemaProperty {
  type: string;
  description: string;
  defaultValue?: string | null;
}

export interface McpToolDescriptor {
  name: string;
  description: string;
  inputSchema: {
    type: string;
    properties: Record<string, McpSchemaProperty>;
    required: string[];
  };
  response: McpToolOutput[];
  updatedAt: number;
}

export interface McpServiceStatus {
  managed: boolean;
  state: string;
  reachable: boolean;
  baseURL: string;
  healthURL: string;
  startedAt?: number | null;
  lastError: string;
  message: string;
}

export interface McpTestResult {
  tool: string;
  accepted: boolean;
  validation: string;
  executedAt: number;
  output: Record<string, unknown>;
}

export interface SkillAssetRecord {
  name: string;
  content: string;
  updatedAt: number;
}

export interface SkillRecord {
  name: string;
  description: string;
  allowedTools: string;
  context: string;
  agent: string;
  markdown: string;
  enabled: boolean;
  tags: string[];
  createdAt: number;
  updatedAt: number;
  scripts: SkillAssetRecord[];
  resources: SkillAssetRecord[];
  references: SkillAssetRecord[];
  templates: SkillAssetRecord[];
}

export interface SkillSummary {
  name: string;
  description: string;
  enabled: boolean;
  tags: string[];
  updatedAt: number;
}

export interface SkillToolDefinition {
  key: string;
  label: string;
  summary: string;
}

export interface GlobalConstraintRecord {
  content: string;
  updatedAt: number;
}

export interface OperationLogInfo {
  ID: number;
  ip: string;
  method: string;
  path: string;
  status: number;
  latency: number;
}

export interface LoginLogInfo {
  ID: number;
  username: string;
  ip: string;
  status: boolean;
  errorMessage: string;
}

export interface DictionaryDetailInfo {
  ID: number;
  label: string;
  value: string;
  extend: string;
  level: number;
  status: boolean;
  sort: number;
  parentID?: number | null;
  children: DictionaryDetailInfo[];
}

export interface ErrorLogInfo {
  ID: number;
  error: string;
  path: string;
  status: string;
}

export interface ExportTemplateInfo {
  ID: number;
  name: string;
  template_id: string;
  desc: string;
}

export interface RuntimeInfo {
  os: string;
  cpuCores: number;
  rustVersion: string;
  dbBackend: string;
  redisEnabled: boolean;
}

export interface RuntimeInfoResult {
  server: RuntimeInfo;
}

export interface SystemConfigInfo {
  bindAddress: string;
  databaseUrl: string;
  redisUrl: string;
  multipointEnabled: boolean;
  compatibilityRefreshHeaders: boolean;
}
