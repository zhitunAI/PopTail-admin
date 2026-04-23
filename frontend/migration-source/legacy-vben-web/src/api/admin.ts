import { apiClient } from "./client";
import type {
  AnnouncementDataSource,
  AnnouncementRecord,
  ApiInfo,
  ApiTokenRecord,
  ApiUpsertInput,
  ApiResponse,
  AutoCodeRegistryRecord,
  AuthorityUpsertInput,
  AuthorityInfo,
  CustomerRecord,
  DictionaryDetailUpsertInput,
  DictionaryInfo,
  DictionaryUpsertInput,
  DictionaryDetailInfo,
  ErrorLogInfo,
  EmailRecord,
  ExportTemplateInfo,
  GlobalConstraintRecord,
  LoginLogInfo,
  McpServiceStatus,
  McpTestResult,
  McpToolDescriptor,
  McpToolOutput,
  McpToolParam,
  McpToolRecord,
  MenuTreeResult,
  OperationLogInfo,
  PageResult,
  PackageRecord,
  ParamInfo,
  PluginInstallRecord,
  PluginManifestRecord,
  ProfileUpdateInput,
  ReleaseRecord,
  RuntimeInfoResult,
  ResumeUploadRecord,
  ScanSessionRecord,
  SkillAssetRecord,
  SkillRecord,
  SkillSummary,
  SkillToolDefinition,
  SystemConfigInfo,
  UploadFileRecord,
  UploadQueueMutationInput,
  MenuUpsertInput,
  UserInfo,
  UserUpsertInput,
} from "../types";

export async function getUserListApi(filters: Record<string, unknown> = {}) {
  const { data } = await apiClient.post<ApiResponse<PageResult<UserInfo>>>(
    "/user/getUserList",
    {
      page: 1,
      pageSize: 50,
      ...filters,
    },
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "获取用户列表失败");
  }
  return data.data;
}

export async function emailTestApi(payload: {
  to?: string;
  subject?: string;
  body?: string;
}) {
  const { data } = await apiClient.post<ApiResponse<EmailRecord>>("/email/emailTest", payload);
  if (data.code !== 0) {
    throw new Error(data.msg || "发送测试邮件失败");
  }
  return data.data;
}

export async function getEmailListApi() {
  const { data } = await apiClient.get<ApiResponse<PageResult<EmailRecord>>>("/email/getEmailList");
  if (data.code !== 0) {
    throw new Error(data.msg || "获取邮件记录失败");
  }
  return data.data;
}

export async function sendEmailApi(payload: {
  to: string;
  subject: string;
  body: string;
}) {
  const { data } = await apiClient.post<ApiResponse<EmailRecord>>("/email/sendEmail", payload);
  if (data.code !== 0) {
    throw new Error(data.msg || "发送邮件失败");
  }
  return data.data;
}

export async function getAnnouncementListApi(filters: Record<string, unknown> = {}) {
  const { data } = await apiClient.get<ApiResponse<PageResult<AnnouncementRecord>>>(
    "/info/getInfoList",
    { params: { page: 1, pageSize: 50, ...filters } },
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "获取公告列表失败");
  }
  return data.data;
}

export async function getAnnouncementApi(ID: number) {
  const { data } = await apiClient.get<ApiResponse<AnnouncementRecord>>("/info/findInfo", {
    params: { ID },
  });
  if (data.code !== 0) {
    throw new Error(data.msg || "获取公告失败");
  }
  return data.data;
}

export async function getAnnouncementDataSourceApi() {
  const { data } = await apiClient.get<ApiResponse<AnnouncementDataSource>>(
    "/info/getInfoDataSource",
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "获取公告数据源失败");
  }
  return data.data;
}

export async function createAnnouncementApi(payload: {
  ID?: number;
  title: string;
  content: string;
  userID: number;
  attachments: { name: string; url: string }[];
}) {
  const { data } = await apiClient.post<ApiResponse<AnnouncementRecord>>("/info/createInfo", payload);
  if (data.code !== 0) {
    throw new Error(data.msg || "创建公告失败");
  }
  return data.data;
}

export async function updateAnnouncementApi(payload: {
  ID?: number;
  title: string;
  content: string;
  userID: number;
  attachments: { name: string; url: string }[];
}) {
  const { data } = await apiClient.put<ApiResponse<AnnouncementRecord>>("/info/updateInfo", payload);
  if (data.code !== 0) {
    throw new Error(data.msg || "更新公告失败");
  }
  return data.data;
}

export async function deleteAnnouncementApi(ID: number) {
  const { data } = await apiClient.delete<ApiResponse<Record<string, never>>>("/info/deleteInfo", {
    params: { ID },
  });
  if (data.code !== 0) {
    throw new Error(data.msg || "删除公告失败");
  }
}

export async function deleteAnnouncementByIdsApi(IDs: number[]) {
  const { data } = await apiClient.delete<ApiResponse<Record<string, unknown>>>(
    "/info/deleteInfoByIds",
    { params: { IDs } },
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "批量删除公告失败");
  }
  return data.data;
}

export async function getAuthorityListApi() {
  const { data } = await apiClient.post<ApiResponse<AuthorityInfo[]>>(
    "/authority/getAuthorityList",
    {},
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "获取角色列表失败");
  }
  return data.data;
}

export async function saveAuthorityApi(payload: AuthorityUpsertInput) {
  const { data } = await apiClient.post<ApiResponse<AuthorityInfo>>(
    "/authority/saveAuthority",
    payload,
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "保存角色失败");
  }
  return data.data;
}

export async function getMenuTreeApi() {
  const { data } = await apiClient.post<ApiResponse<MenuTreeResult>>("/menu/getMenu", {});
  if (data.code !== 0) {
    throw new Error(data.msg || "获取菜单树失败");
  }
  return data.data.menus;
}

export async function getMenuListApi() {
  const { data } = await apiClient.post<ApiResponse<PageResult<any>>>("/menu/getMenuList", {});
  if (data.code !== 0) {
    throw new Error(data.msg || "获取菜单列表失败");
  }
  return data.data;
}

export async function saveMenuApi(payload: MenuUpsertInput) {
  const { data } = await apiClient.post<ApiResponse<any>>("/menu/saveMenu", payload);
  if (data.code !== 0) {
    throw new Error(data.msg || "保存菜单失败");
  }
  return data.data;
}

export async function getApiListApi(filters: Record<string, unknown> = {}) {
  const { data } = await apiClient.post<ApiResponse<PageResult<ApiInfo>>>(
    "/api/getApiList",
    {
      page: 1,
      pageSize: 50,
      ...filters,
    },
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "获取接口列表失败");
  }
  return data.data;
}

export async function saveApiApi(payload: ApiUpsertInput) {
  const { data } = await apiClient.post<ApiResponse<ApiInfo>>("/api/saveApi", payload);
  if (data.code !== 0) {
    throw new Error(data.msg || "保存接口失败");
  }
  return data.data;
}

export async function getDictionaryListApi() {
  const { data } = await apiClient.get<ApiResponse<PageResult<DictionaryInfo>>>(
    "/sysDictionary/getSysDictionaryList",
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "获取字典列表失败");
  }
  return data.data;
}

export async function saveDictionaryApi(payload: DictionaryUpsertInput) {
  const { data } = await apiClient.post<ApiResponse<DictionaryInfo>>(
    "/sysDictionary/saveSysDictionary",
    payload,
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "保存字典失败");
  }
  return data.data;
}

export async function getDictionaryDetailTreeApi(sysDictionaryID: number) {
  const { data } = await apiClient.get<ApiResponse<DictionaryDetailInfo[]>>(
    "/sysDictionaryDetail/getDictionaryTreeList",
    {
      params: { sysDictionaryID },
    },
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "获取字典详情失败");
  }
  return data.data;
}

export async function getParamsListApi() {
  const { data } = await apiClient.get<ApiResponse<PageResult<ParamInfo>>>(
    "/sysParams/getSysParamsList",
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "获取参数列表失败");
  }
  return data.data;
}

export async function saveParamApi(payload: {
  ID?: number;
  key: string;
  value: string;
  desc: string;
}) {
  const { data } = await apiClient.post<ApiResponse<ParamInfo>>(
    "/sysParams/saveSysParams",
    payload,
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "保存参数失败");
  }
  return data.data;
}

export async function updateProfileApi(payload: ProfileUpdateInput) {
  const { data } = await apiClient.post<
    ApiResponse<{ userInfo: UserInfo }>
  >("/user/updateProfile", payload);
  if (data.code !== 0) {
    throw new Error(data.msg || "保存个人资料失败");
  }
  return data.data.userInfo;
}

export async function saveUserApi(payload: UserUpsertInput) {
  const { data } = await apiClient.post<
    ApiResponse<{ userInfo: UserInfo }>
  >("/user/saveUser", payload);
  if (data.code !== 0) {
    throw new Error(data.msg || "保存用户失败");
  }
  return data.data.userInfo;
}

export async function saveDictionaryDetailApi(payload: DictionaryDetailUpsertInput) {
  const { data } = await apiClient.post<ApiResponse<DictionaryDetailInfo>>(
    "/sysDictionaryDetail/saveSysDictionaryDetail",
    payload,
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "保存字典详情失败");
  }
  return data.data;
}

export async function getOperationLogsApi() {
  const { data } = await apiClient.get<ApiResponse<PageResult<OperationLogInfo>>>(
    "/sysOperationRecord/getSysOperationRecordList",
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "获取操作日志失败");
  }
  return data.data;
}

export async function getLoginLogsApi() {
  const { data } = await apiClient.get<ApiResponse<PageResult<LoginLogInfo>>>(
    "/sysLoginLog/getLoginLogList",
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "获取登录日志失败");
  }
  return data.data;
}

export async function getErrorLogsApi() {
  const { data } = await apiClient.get<ApiResponse<PageResult<ErrorLogInfo>>>(
    "/sysError/getSysErrorList",
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "获取错误日志失败");
  }
  return data.data;
}

export async function getExportTemplatesApi() {
  const { data } = await apiClient.get<ApiResponse<PageResult<ExportTemplateInfo>>>(
    "/sysExportTemplate/getSysExportTemplateList",
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "获取导出模板失败");
  }
  return data.data;
}

export async function getRuntimeInfoApi() {
  const { data } = await apiClient.post<ApiResponse<RuntimeInfoResult>>(
    "/system/getServerInfo",
    {},
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "获取运行状态失败");
  }
  return data.data.server;
}

export async function getSystemConfigApi() {
  const { data } = await apiClient.post<ApiResponse<SystemConfigInfo>>(
    "/system/getSystemConfig",
    {},
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "获取系统配置失败");
  }
  return data.data;
}

export async function updateSystemConfigApi(payload: SystemConfigInfo) {
  const { data } = await apiClient.post<ApiResponse<SystemConfigInfo>>(
    "/system/setSystemConfig",
    payload,
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "保存系统配置失败");
  }
  return data.data;
}

export async function submitAiModerationDecision(payload: {
  mode: "system" | "delegated";
  articleId: string;
  action: string;
  reason: string;
  serviceToken?: string;
  delegation?: {
    operatorUserId: number;
    scope: string;
  };
}) {
  const { data } = await apiClient.post<
    ApiResponse<{ accepted: boolean; auditId: string }>
  >("/ai/moderation/decision", payload);
  if (data.code !== 0) {
    throw new Error(data.msg || "提交 AI 处置失败");
  }
  return data.data;
}

export async function getApiTokenListApi() {
  const { data } = await apiClient.get<ApiResponse<PageResult<ApiTokenRecord>>>(
    "/tool/api-token/list",
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "获取令牌台账失败");
  }
  return data.data;
}

export async function issueApiTokenApi(payload: {
  name: string;
  scope: string;
  ttl: string;
}) {
  const { data } = await apiClient.post<ApiResponse<ApiTokenRecord>>(
    "/tool/api-token/issue",
    payload,
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "登记令牌失败");
  }
  return data.data;
}

export async function clearApiTokenApi() {
  const { data } = await apiClient.post<ApiResponse<Record<string, never>>>(
    "/tool/api-token/clear",
    {},
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "清空令牌失败");
  }
}

export async function getPackageListApi() {
  const { data } = await apiClient.get<ApiResponse<PageResult<PackageRecord>>>(
    "/tool/package/list",
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "获取包定义失败");
  }
  return data.data;
}

export async function savePackageApi(payload: {
  ID?: number;
  name: string;
  kind: string;
  output: string;
  summary: string;
}) {
  const { data } = await apiClient.post<ApiResponse<PackageRecord>>(
    "/tool/package/save",
    payload,
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "保存包定义失败");
  }
  return data.data;
}

export async function getPluginManifestListApi() {
  const { data } = await apiClient.get<ApiResponse<PageResult<PluginManifestRecord>>>(
    "/tool/plugin-manifest/list",
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "获取插件清单失败");
  }
  return data.data;
}

export async function savePluginManifestApi(payload: {
  pluginName: string;
  menuGroup: string;
  menuIds: number[];
  apiIds: number[];
  dictionaryIds: number[];
}) {
  const { data } = await apiClient.post<ApiResponse<PluginManifestRecord>>(
    "/tool/plugin-manifest/save",
    payload,
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "保存插件清单失败");
  }
  return data.data;
}

export async function getPluginInstallListApi() {
  const { data } = await apiClient.get<ApiResponse<PageResult<PluginInstallRecord>>>(
    "/tool/plugin-install/list",
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "获取插件安装记录失败");
  }
  return data.data;
}

export async function savePluginInstallApi(payload: {
  fileName: string;
  kind: string;
  target: string;
  manifest: string;
}) {
  const { data } = await apiClient.post<ApiResponse<PluginInstallRecord>>(
    "/tool/plugin-install/save",
    payload,
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "登记插件安装失败");
  }
  return data.data;
}

export async function getAutoCodeRegistryApi() {
  const { data } = await apiClient.get<ApiResponse<PageResult<AutoCodeRegistryRecord>>>(
    "/tool/auto-code/list",
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "获取自动代码台账失败");
  }
  return data.data;
}

export async function saveAutoCodeRegistryApi(payload: Record<string, unknown>) {
  const { data } = await apiClient.post<ApiResponse<AutoCodeRegistryRecord>>(
    "/tool/auto-code/save",
    { payload },
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "保存自动代码台账失败");
  }
  return data.data;
}

export async function clearAutoCodeRegistryApi() {
  const { data } = await apiClient.post<ApiResponse<Record<string, never>>>(
    "/tool/auto-code/clear",
    {},
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "清空自动代码台账失败");
  }
}

export async function getReleaseListApi() {
  const { data } = await apiClient.get<ApiResponse<PageResult<ReleaseRecord>>>(
    "/tool/release/list",
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "获取发布记录失败");
  }
  return data.data;
}

export async function saveReleaseApi(payload: { note: string }) {
  const { data } = await apiClient.post<ApiResponse<ReleaseRecord>>(
    "/tool/release/save",
    payload,
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "保存发布记录失败");
  }
  return data.data;
}

export async function saveUploadQueueApi(payload: UploadQueueMutationInput = {}) {
  const { data } = await apiClient.post<ApiResponse<PageResult<UploadFileRecord>>>(
    "/example/upload/list",
    {
      files: payload.files ?? [],
      mode: payload.mode ?? "replace",
    },
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "保存上传队列失败");
  }
  return data.data;
}

export async function getUploadQueueApi() {
  return saveUploadQueueApi({ mode: "inspect" });
}

export async function completeUploadQueueApi() {
  const { data } = await apiClient.post<ApiResponse<PageResult<UploadFileRecord>>>(
    "/example/upload/complete",
    {},
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "更新上传状态失败");
  }
  return data.data;
}

export async function getResumeUploadListApi() {
  const { data } = await apiClient.get<ApiResponse<PageResult<ResumeUploadRecord>>>(
    "/example/resume/list",
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "获取续传任务失败");
  }
  return data.data;
}

export async function advanceResumeUploadApi() {
  const { data } = await apiClient.post<ApiResponse<PageResult<ResumeUploadRecord>>>(
    "/example/resume/advance",
    {},
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "推进续传失败");
  }
  return data.data;
}

export async function recoverResumeUploadApi() {
  const { data } = await apiClient.post<ApiResponse<PageResult<ResumeUploadRecord>>>(
    "/example/resume/recover",
    {},
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "恢复续传失败");
  }
  return data.data;
}

export async function getScanSessionApi() {
  const { data } = await apiClient.get<ApiResponse<ScanSessionRecord>>(
    "/example/scan/session",
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "获取扫码会话失败");
  }
  return data.data;
}

export async function saveScanSessionApi(payload: ScanSessionRecord) {
  const { data } = await apiClient.post<ApiResponse<ScanSessionRecord>>(
    "/example/scan/session",
    payload,
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "保存扫码会话失败");
  }
  return data.data;
}

export async function getCustomerListApi(filters: Record<string, unknown> = {}) {
  const { data } = await apiClient.get<ApiResponse<PageResult<CustomerRecord>>>(
    "/customer/customerList",
    {
      params: {
        page: 1,
        pageSize: 20,
        ...filters,
      },
    },
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "获取客户列表失败");
  }
  return data.data;
}

export async function getCustomerDetailApi(ID: number) {
  const { data } = await apiClient.get<ApiResponse<{ customer: CustomerRecord }>>(
    "/customer/customer",
    {
      params: { ID },
    },
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "获取客户详情失败");
  }
  return data.data.customer;
}

export async function createCustomerApi(payload: {
  customerName: string;
  customerPhoneData: string;
  customerLevel?: string;
  customerStatus?: string;
  remark?: string;
}) {
  const { data } = await apiClient.post<ApiResponse<{ customer: CustomerRecord }>>(
    "/customer/customer",
    payload,
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "创建客户失败");
  }
  return data.data.customer;
}

export async function updateCustomerApi(payload: {
  ID: number;
  customerName: string;
  customerPhoneData: string;
  customerLevel?: string;
  customerStatus?: string;
  remark?: string;
}) {
  const { data } = await apiClient.put<ApiResponse<{ customer: CustomerRecord }>>(
    "/customer/customer",
    payload,
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "更新客户失败");
  }
  return data.data.customer;
}

export async function deleteCustomerApi(ID: number) {
  const { data } = await apiClient.delete<ApiResponse<Record<string, never>>>(
    "/customer/customer",
    {
      data: { ID },
    },
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "删除客户失败");
  }
}

export async function saveMcpToolApi(payload: {
  name: string;
  description: string;
  params: McpToolParam[];
  response: McpToolOutput[];
}) {
  const { data } = await apiClient.post<ApiResponse<McpToolRecord>>("/autoCode/mcp", payload);
  if (data.code !== 0) {
    throw new Error(data.msg || "保存 MCP 工具失败");
  }
  return data.data;
}

export async function getMcpStatusApi() {
  const { data } = await apiClient.post<ApiResponse<McpServiceStatus>>(
    "/autoCode/mcpStatus",
    {},
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "获取 MCP 状态失败");
  }
  return data.data;
}

export async function startMcpServiceApi() {
  const { data } = await apiClient.post<ApiResponse<McpServiceStatus>>(
    "/autoCode/mcpStart",
    {},
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "启动 MCP 服务失败");
  }
  return data.data;
}

export async function stopMcpServiceApi() {
  const { data } = await apiClient.post<ApiResponse<McpServiceStatus>>(
    "/autoCode/mcpStop",
    {},
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "停用 MCP 服务失败");
  }
  return data.data;
}

export async function getMcpToolListApi() {
  const { data } = await apiClient.post<ApiResponse<{ tools: McpToolDescriptor[] }>>(
    "/autoCode/mcpList",
    {},
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "获取 MCP 工具列表失败");
  }
  return data.data.tools;
}

export async function testMcpToolApi(payload: {
  name: string;
  args: Record<string, unknown>;
}) {
  const { data } = await apiClient.post<ApiResponse<McpTestResult>>("/autoCode/mcpTest", payload);
  if (data.code !== 0) {
    throw new Error(data.msg || "执行 MCP 测试失败");
  }
  return data.data;
}

export async function getSkillToolsApi() {
  const { data } = await apiClient.get<ApiResponse<{ tools: SkillToolDefinition[] }>>(
    "/skills/getTools",
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "获取工具清单失败");
  }
  return data.data.tools;
}

export async function getSkillListApi() {
  const { data } = await apiClient.post<ApiResponse<{ skills: SkillSummary[] }>>(
    "/skills/getSkillList",
    {},
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "获取技能列表失败");
  }
  return data.data.skills;
}

export async function getSkillDetailApi(name: string) {
  const { data } = await apiClient.post<ApiResponse<{ skill: SkillRecord }>>(
    "/skills/getSkillDetail",
    { name },
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "获取技能详情失败");
  }
  return data.data.skill;
}

export async function saveSkillApi(payload: {
  name: string;
  description: string;
  allowedTools: string;
  context: string;
  agent: string;
  markdown: string;
  enabled: boolean;
  tags: string[];
}) {
  const { data } = await apiClient.post<ApiResponse<{ skill: SkillRecord }>>(
    "/skills/saveSkill",
    payload,
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "保存技能失败");
  }
  return data.data.skill;
}

export async function deleteSkillApi(name: string) {
  const { data } = await apiClient.post<ApiResponse<Record<string, never>>>(
    "/skills/deleteSkill",
    { name },
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "删除技能失败");
  }
}

function assetEndpoint(kind: "script" | "resource" | "reference" | "template") {
  switch (kind) {
    case "script":
      return {
        create: "/skills/createScript",
        get: "/skills/getScript",
        save: "/skills/saveScript",
      };
    case "resource":
      return {
        create: "/skills/createResource",
        get: "/skills/getResource",
        save: "/skills/saveResource",
      };
    case "reference":
      return {
        create: "/skills/createReference",
        get: "/skills/getReference",
        save: "/skills/saveReference",
      };
    default:
      return {
        create: "/skills/createTemplate",
        get: "/skills/getTemplate",
        save: "/skills/saveTemplate",
      };
  }
}

export async function createSkillAssetApi(
  kind: "script" | "resource" | "reference" | "template",
  payload: { skillName: string; name: string },
) {
  const endpoint = assetEndpoint(kind).create;
  const { data } = await apiClient.post<ApiResponse<{ file: SkillAssetRecord }>>(endpoint, payload);
  if (data.code !== 0) {
    throw new Error(data.msg || "创建文件失败");
  }
  return data.data.file;
}

export async function getSkillAssetApi(
  kind: "script" | "resource" | "reference" | "template",
  payload: { skillName: string; name: string },
) {
  const endpoint = assetEndpoint(kind).get;
  const { data } = await apiClient.post<ApiResponse<{ file: SkillAssetRecord }>>(endpoint, payload);
  if (data.code !== 0) {
    throw new Error(data.msg || "获取文件失败");
  }
  return data.data.file;
}

export async function saveSkillAssetApi(
  kind: "script" | "resource" | "reference" | "template",
  payload: { skillName: string; name: string; content: string },
) {
  const endpoint = assetEndpoint(kind).save;
  const { data } = await apiClient.post<ApiResponse<{ file: SkillAssetRecord }>>(endpoint, payload);
  if (data.code !== 0) {
    throw new Error(data.msg || "保存文件失败");
  }
  return data.data.file;
}

export async function getGlobalConstraintApi() {
  const { data } = await apiClient.post<ApiResponse<{ globalConstraint: GlobalConstraintRecord }>>(
    "/skills/getGlobalConstraint",
    {},
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "获取全局约束失败");
  }
  return data.data.globalConstraint;
}

export async function saveGlobalConstraintApi(content: string) {
  const { data } = await apiClient.post<ApiResponse<{ globalConstraint: GlobalConstraintRecord }>>(
    "/skills/saveGlobalConstraint",
    { content },
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "保存全局约束失败");
  }
  return data.data.globalConstraint;
}
