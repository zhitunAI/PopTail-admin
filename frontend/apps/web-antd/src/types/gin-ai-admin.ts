export interface AuthorityInfo {
  ID: number;
  authorityId: number;
  authorityName: string;
  defaultRouter: string;
  enable?: number;
  parentId: number;
  status?: number;
  children: AuthorityInfo[];
  menus?: MenuInfo[];
  userIds?: number[];
}

export interface UserInfo {
  ID: number;
  uuid: string;
  userName: string;
  nickName: string;
  authorityId: number;
  authority: AuthorityInfo;
  authorities: AuthorityInfo[];
  authorityIds?: number[];
  headerImg: string;
  phone: string;
  email: string;
  enable: number;
  originSetting?: Record<string, unknown>;
}

export interface ApiResponse<T> {
  code: number;
  msg: string;
  data: T;
}

export type MutationAck = Record<string, never>;

export interface IdInput {
  ID: number;
}

export interface IdsInput {
  IDs: number[];
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
  activeName?: string;
  closeTab?: boolean;
  defaultMenu?: boolean;
  keepAlive?: boolean;
  transitionType?: string;
  btns?: Record<string, boolean>;
}

export interface MenuParameterInfo {
  ID?: number;
  sysBaseMenuID?: number;
  type: string;
  key: string;
  value: string;
}

export interface MenuButtonInfo {
  ID?: number;
  sysBaseMenuID?: number;
  name: string;
  desc: string;
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
  authoritys?: AuthorityInfo[];
  menuBtn?: MenuButtonInfo[];
  parameters?: MenuParameterInfo[];
}

export interface MenuTreeResult {
  menus: MenuInfo[];
}

export interface ApiInfo {
  ID: number;
  CreatedAt?: number | string;
  UpdatedAt?: number | string;
  path: string;
  apiGroup: string;
  description: string;
  method: string;
}

export interface ApiSearchInput {
  page?: number;
  pageSize?: number;
  keyword?: string;
  path?: string;
  apiGroup?: string;
  description?: string;
  method?: string;
  orderKey?: string;
  desc?: boolean;
}

export interface ApiDeleteInput extends IdInput {}

export interface ApiDeleteIdsInput {
  ids: number[];
}

export interface ApiGroupResult {
  apiGroupMap: Record<string, string[]>;
  groups: string[];
}

export interface ApiIgnoreRecord {
  ID?: number;
  path: string;
  method: string;
  flag?: boolean;
}

export interface ApiSyncPreviewResult {
  newApis: ApiInfo[];
  deleteApis: ApiInfo[];
  ignoreApis: ApiIgnoreRecord[];
}

export interface ApiSyncCommitInput {
  newApis: ApiInfo[];
  deleteApis: ApiInfo[];
  ignoreApis?: ApiIgnoreRecord[];
}

export interface ApiRoleBindingQuery {
  path: string;
  method: string;
}

export interface ApiRoleBindingInput extends ApiRoleBindingQuery {
  authorityIds: number[];
}

export interface DictionaryInfo {
  ID: number;
  name: string;
  type: string;
  status: boolean;
  desc: string;
  parentID?: null | number;
  children?: DictionaryInfo[];
  sysDictionaryDetails?: DictionaryDetailInfo[];
}

export interface ParamInfo {
  ID: number;
  name?: string;
  key: string;
  value: string;
  desc: string;
  CreatedAt?: number | string;
  UpdatedAt?: number | string;
}

export interface LlmConfigInfo {
  ID: number;
  apiKey: string;
  baseUrl: string;
  model: string;
  provider: string;
  status: string;
}

export interface FrontendNavInfo {
  ID: number;
  icon: string;
  order: number;
  path: string;
  title: string;
  visible: boolean;
}

export interface FrontendSettingsInfo {
  logoUrl: string;
  mailFrom: string;
  recordNumber: string;
  siteDescription: string;
  siteName: string;
  siteSlogan: string;
  smtpHost: string;
  smtpPassword: string;
  smtpPort: number;
  smtpUser: string;
}

export interface ArticleCategoryInfo {
  ID: number;
  name: string;
  slug: string;
  sort: number;
  status: boolean;
}

export interface ArticleInfo {
  ID: number;
  categoryId: number;
  content: string;
  slug: string;
  status: string;
  title: string;
  updatedAt: string;
}

export interface MemberInfo {
  ID: number;
  consolePath: string;
  email: string;
  lastLogin: string;
  nickname: string;
  provider: string;
  status: string;
}

export interface ConsoleMenuInfo {
  ID: number;
  group: string;
  icon: string;
  order: number;
  path: string;
  title: string;
  visible: boolean;
}

export interface ParamSearchInput {
  page?: number;
  pageSize?: number;
  keyword?: string;
  key?: string;
  name?: string;
  startCreatedAt?: number | string;
  endCreatedAt?: number | string;
}

export interface ParamValueQuery {
  key: string;
}

export interface ParamHistoryQuery extends ParamValueQuery {}

export interface ParamHistoryRecord {
  ID?: number;
  key: string;
  value: string;
  desc?: string;
  changedAt?: number | string;
  changedBy?: string;
  note?: string;
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
  authorityIds?: number[];
  phone: string;
  email: string;
  enable: number;
  headerImg?: string;
}

export interface UserDeleteInput extends IdInput {}

export interface UserResetPasswordInput extends IdInput {
  password: string;
}

export interface UserAuthorityAssignmentInput extends IdInput {
  authorityIds: number[];
}

export interface AuthorityUpsertInput {
  ID?: number;
  authorityId: number;
  authorityName: string;
  defaultRouter: string;
  enable?: number;
  parentId: number;
  status?: number;
}

export interface AuthorityCopyInput {
  authority: AuthorityInfo | AuthorityUpsertInput;
  oldAuthorityId: number;
}

export interface AuthorityCopyResult {
  authority: AuthorityInfo;
  oldAuthorityId: number;
}

export interface AuthorityRoleUsersInput {
  authorityId: number;
  userIds: number[];
}

export interface AuthorityButtonMatrixInput {
  menuID: number;
  authorityId: number;
  selected?: number[];
}

export interface AuthorityButtonMatrixSelection {
  selected: number[];
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
  parentID?: null | number;
}

export interface DictionarySearchInput {
  page?: number;
  pageSize?: number;
  keyword?: string;
  name?: string;
  type?: string;
  status?: boolean;
  parentID?: null | number;
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
  menuBtn?: MenuButtonInfo[];
  parameters?: MenuParameterInfo[];
}

export interface MenuAuthorityAssignmentInput {
  authorityId: number;
  menus: MenuInfo[];
}

export interface MenuRoleIdsResult {
  authorityIds: number[];
  defaultRouterAuthorityIds: number[];
}

export interface MenuRoleUpdateInput {
  menuId: number;
  authorityIds: number[];
}

export interface ApiUpsertInput {
  ID?: number;
  path: string;
  apiGroup: string;
  description: string;
  method: string;
}

export interface ParamUpsertInput {
  ID?: number;
  name?: string;
  key: string;
  value: string;
  desc: string;
}

export interface ParamDeleteInput extends IdInput {}

export interface ParamDeleteIdsInput {
  ids?: number[];
  IDs?: number[];
  'IDs[]'?: number[];
}

export interface DictionaryUpsertInput {
  ID?: number;
  name: string;
  type: string;
  status: boolean;
  desc: string;
  parentID?: null | number;
}

export interface DictionaryDeleteInput extends IdInput {}

export interface DictionaryDeleteIdsInput {
  ids?: number[];
  IDs?: number[];
}

export interface DictionaryBatchInput {
  items: DictionaryUpsertInput[];
}

export interface DictionaryImportInput {
  json: string;
}

export interface DictionaryExportRecord {
  name: string;
  type: string;
  status?: boolean | null;
  desc: string;
  sysDictionaryDetails: DictionaryDetailInfo[];
}

export interface ApiTokenIssueInput {
  userId?: number;
  authorityId?: number;
  name: string;
  scope: string;
  ttl: string;
  remark?: string;
}

export interface ApiTokenRecord {
  ID: number;
  name: string;
  scope: string;
  ttl: string;
  status: string;
  authorityId?: number;
  expiresAt?: null | number | string;
  lastUsedAt?: null | number | string;
  remark?: string;
  token?: string;
  user?: Pick<UserInfo, 'ID' | 'nickName' | 'userName'>;
  userId?: number;
}

export interface ApiTokenDetailRecord extends ApiTokenRecord {
  curlPayload?: ApiTokenCurlPayload;
}

export interface ApiTokenCurlPayload {
  command?: string;
  cookie?: string;
  header: string;
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
    [key: string]: unknown;
    entity: string;
    module: string;
    outputs: string[];
    table: string;
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

export interface UploadedFileAsset {
  ID?: number;
  classId?: number;
  key?: string;
  name: string;
  url: string;
}

export interface UploadQueueMutationInput {
  files?: Array<{
    name: string;
    size: string;
  }>;
  mode?: "append" | "inspect" | "replace";
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
  defaultValue?: null | string;
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
  defaultValue?: null | string;
}

export interface McpToolDescriptor {
  name: string;
  description: string;
  inputSchema: {
    properties: Record<string, McpSchemaProperty>;
    required: string[];
    type: string;
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
  startedAt?: null | number;
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
  CreatedAt?: number | string;
  UpdatedAt?: number | string;
  ip: string;
  method: string;
  path: string;
  status: number;
  latency: number;
  agent?: string;
  errorMessage?: string;
  body?: string;
  resp?: string;
  userId?: number;
  user?: Pick<UserInfo, 'ID' | 'nickName' | 'userName'>;
}

export interface LoginLogInfo {
  ID: number;
  CreatedAt?: number | string;
  UpdatedAt?: number | string;
  username: string;
  ip: string;
  status: boolean;
  errorMessage: string;
  agent?: string;
  userId?: number;
  user?: Pick<UserInfo, 'ID' | 'nickName' | 'userName'>;
}

export interface DictionaryDetailInfo {
  ID: number;
  CreatedAt?: number | string;
  UpdatedAt?: number | string;
  sysDictionaryID?: number;
  label: string;
  value: string;
  extend: string;
  level: number;
  status: boolean;
  sort: number;
  parentID?: null | number;
  children: DictionaryDetailInfo[];
}

export interface DictionaryDetailSearchInput {
  page?: number;
  pageSize?: number;
  keyword?: string;
  sysDictionaryID?: number;
  parentID?: null | number;
  level?: null | number;
  label?: string;
  value?: string;
}

export interface DictionaryDetailDeleteInput extends IdInput {}

export interface DictionaryDetailDeleteIdsInput {
  ids?: number[];
  IDs?: number[];
}

export interface DictionaryTreeByTypeQuery {
  type: string;
}

export interface DictionaryDetailsByParentInput {
  sysDictionaryID: number;
  parentID?: null | number;
  includeChildren?: boolean;
}

export interface DictionaryPathResult {
  labels?: string[];
  path?: DictionaryDetailInfo[];
}

export interface DictionaryDetailReorderItem {
  ID: number;
  parentID?: null | number;
  sort: number;
}

export interface DictionaryDetailReorderInput {
  sysDictionaryID: number;
  items: DictionaryDetailReorderItem[];
}

export interface LoginLogSearchInput {
  page?: number;
  pageSize?: number;
  keyword?: string;
  username?: string;
  ip?: string;
  status?: boolean;
  startCreatedAt?: number | string;
  endCreatedAt?: number | string;
}

export interface LoginLogDeleteInput extends IdInput {}

export interface LoginLogDeleteIdsInput {
  ids: number[];
}

export interface LoginLogExportQuery extends LoginLogSearchInput {}

export interface OperationLogSearchInput {
  page?: number;
  pageSize?: number;
  keyword?: string;
  method?: string;
  path?: string;
  ip?: string;
  status?: number;
  userId?: number;
  startCreatedAt?: number | string;
  endCreatedAt?: number | string;
}

export interface OperationLogDeleteInput extends IdInput {}

export interface OperationLogDeleteIdsInput {
  ids: number[];
}

export interface OperationLogExportQuery extends OperationLogSearchInput {}

export interface BlobArtifact {
  fileName?: string;
  mimeType?: string;
  blob?: Blob;
  url?: string;
}

export interface ErrorLogInfo {
  ID: number;
  error: string;
  path: string;
  status: string;
  form?: string;
  level?: string;
  solution?: string;
  createdAt?: number | string;
  updatedAt?: number | string;
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

export interface SystemConfigSection {
  addr?: number;
  'db-type'?: string;
  'disable-auto-migrate'?: boolean;
  'iplimit-count'?: number;
  'iplimit-time'?: number;
  'oss-type'?: string;
  'router-prefix'?: string;
  'use-mongo'?: boolean;
  'use-multipoint'?: boolean;
  'use-redis'?: boolean;
  'use-strict-auth'?: boolean;
}

export interface SystemConfigJwtSection {
  'buffer-time'?: string;
  'expires-time'?: string;
  issuer?: string;
  'signing-key'?: string;
}

export interface SystemConfigZapSection {
  director?: string;
  'encode-level'?: string;
  format?: string;
  level?: string;
  'log-in-console'?: boolean;
  prefix?: string;
  'retention-day'?: number;
  'show-line'?: boolean;
  'stacktrace-key'?: string;
}

export interface SystemConfigRedisSection {
  addr?: string;
  clusterAddrs?: string[];
  db?: number;
  name?: string;
  password?: string;
  useCluster?: boolean;
}

export interface SystemConfigEmailSection {
  from?: string;
  host?: string;
  'is-loginauth'?: boolean;
  'is-ssl'?: boolean;
  nickname?: string;
  port?: number;
  secret?: string;
  to?: string;
}

export interface SystemConfigInfo {
  bindAddress: string;
  databaseUrl: string;
  redisUrl: string;
  multipointEnabled: boolean;
  compatibilityRefreshHeaders: boolean;
}

export interface SystemConfigSnapshot extends SystemConfigInfo {
  system?: SystemConfigSection;
  jwt?: SystemConfigJwtSection;
  zap?: SystemConfigZapSection;
  redis?: SystemConfigRedisSection;
  email?: SystemConfigEmailSection;
}

export interface ReloadSystemInput {
  force?: boolean;
}

export interface AiWorkflowMessageRecord {
  id: string;
  role: string;
  content: string;
  snapshot: Record<string, unknown>;
  conversationId: string;
  messageId: string;
  createdAt: string;
  parentId?: null | string;
  stage?: string;
  canRollback?: boolean;
}

export interface AiWorkflowSessionListItem {
  ID: number;
  CreatedAt: number | string;
  UpdatedAt: number | string;
  tab: string;
  title: string;
  summary: string;
  conversationId: string;
  currentNodeId: string;
  currentStage?: string;
}

export interface AiWorkflowSessionRecord {
  ID: number;
  CreatedAt?: number | string;
  UpdatedAt?: number | string;
  userId?: number;
  tab: string;
  title: string;
  summary: string;
  conversationId: string;
  messageId: string;
  currentNodeId: string;
  currentStage?: string;
  settings: Record<string, unknown>;
  formData: Record<string, unknown>;
  resultData: Record<string, unknown>;
  messages: AiWorkflowMessageRecord[];
}

export interface AiWorkflowSessionUpsertInput {
  id?: number;
  tab: string;
  title?: string;
  summary?: string;
  conversationId?: string;
  messageId?: string;
  currentNodeId?: string;
  currentStage?: string;
  settings?: Record<string, unknown>;
  formData?: Record<string, unknown>;
  resultData?: Record<string, unknown>;
  messages?: AiWorkflowMessageRecord[];
}

export interface AiWorkflowSessionSearchInput {
  page?: number;
  pageSize?: number;
  tab?: string;
}

export interface AiWorkflowNodeRollbackInput {
  id?: number;
  sessionId?: number;
  nodeId?: string;
  messageId?: string;
}

export interface AiWorkflowResumeChatInput {
  sessionId: number;
  nodeId?: string;
  messageId?: string;
  prompt?: string;
  tab?: string;
}

export interface AiWorkflowStageUpdateInput {
  sessionId: number;
  currentStage: string;
  nodeId?: string;
}

export interface AiWorkflowMarkdownDumpResult {
  directory: string;
  fileName: string;
  filePath: string;
  relativePath: string;
}
