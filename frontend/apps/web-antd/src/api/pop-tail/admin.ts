import type {
  AiWorkflowMarkdownDumpResult,
  AiWorkflowNodeRollbackInput,
  AiWorkflowResumeChatInput,
  AiWorkflowSessionListItem,
  AiWorkflowSessionRecord,
  AiWorkflowSessionSearchInput,
  AiWorkflowSessionUpsertInput,
  AiWorkflowStageUpdateInput,
  AnnouncementDataSource,
  AnnouncementRecord,
  ArticleCategoryInfo,
  ArticleInfo,
  ApiDeleteIdsInput,
  ApiDeleteInput,
  ApiGroupResult,
  ApiIgnoreRecord,
  ApiInfo,
  ApiResponse,
  ApiRoleBindingInput,
  ApiRoleBindingQuery,
  ApiSearchInput,
  ApiSyncCommitInput,
  ApiSyncPreviewResult,
  ApiTokenCurlPayload,
  ApiTokenDetailRecord,
  ApiTokenIssueInput,
  ApiTokenRecord,
  ApiUpsertInput,
  AuthorityButtonMatrixInput,
  AuthorityButtonMatrixBatchInput,
  AuthorityButtonMatrixBatchSelectionItem,
  AuthorityButtonMatrixSelection,
  AuthorityCopyInput,
  AuthorityCopyResult,
  AuthorityInfo,
  AuthorityRoleUsersInput,
  AuthorityUpsertInput,
  AutoCodeRegistryRecord,
  BlobArtifact,
  ConsoleMenuInfo,
  CustomerRecord,
  DictionaryDeleteIdsInput,
  DictionaryDeleteInput,
  DictionaryDetailDeleteIdsInput,
  DictionaryDetailDeleteInput,
  DictionaryDetailInfo,
  DictionaryDetailReorderInput,
  DictionaryDetailsByParentInput,
  DictionaryDetailSearchInput,
  DictionaryDetailUpsertInput,
  DictionaryExportRecord,
  DictionaryImportInput,
  DictionaryInfo,
  DictionaryPathResult,
  DictionarySearchInput,
  DictionaryTreeByTypeQuery,
  DictionaryUpsertInput,
  EmailPresetRecord,
  EmailRecord,
  ErrorLogInfo,
  ExportTemplateInfo,
  FrontendNavInfo,
  FrontendSettingsInfo,
  GlobalConstraintRecord,
  IdInput,
  LlmConfigInfo,
  LoginLogDeleteIdsInput,
  LoginLogDeleteInput,
  LoginLogExportQuery,
  LoginLogInfo,
  LoginLogSearchInput,
  McpServiceStatus,
  McpTestResult,
  McpToolDescriptor,
  McpToolOutput,
  McpToolParam,
  McpToolRecord,
  MemberInfo,
  MenuAuthorityAssignmentInput,
  MenuInfo,
  MenuRoleIdsResult,
  MenuRoleUpdateInput,
  MenuTreeResult,
  MenuUpsertInput,
  MutationAck,
  OperationLogDeleteIdsInput,
  OperationLogDeleteInput,
  OperationLogExportQuery,
  OperationLogInfo,
  OperationLogSearchInput,
  PackageRecord,
  PageResult,
  ParamDeleteIdsInput,
  ParamDeleteInput,
  ParamHistoryQuery,
  ParamHistoryRecord,
  ParamInfo,
  ParamSearchInput,
  ParamUpsertInput,
  ParamValueQuery,
  PluginInstallRecord,
  PluginManifestRecord,
  ProfileUpdateInput,
  ReleaseRecord,
  ReloadSystemInput,
  ResumeUploadRecord,
  RuntimeInfoResult,
  ScanSessionRecord,
  SkillAssetRecord,
  SkillRecord,
  SkillSummary,
  SkillToolDefinition,
  SystemConfigInfo,
  UploadedFileAsset,
  UploadFileRecord,
  UploadQueueMutationInput,
  UserAuthorityAssignmentInput,
  UserDeleteInput,
  UserInfo,
  UserResetPasswordInput,
  UserUpsertInput,
} from "#/types/pop-tail";

import { apiClient } from "./client";

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

export async function uploadFileAssetApi(file: Blob | File, extra: { classId?: number; noSave?: "0" | "1" } = {}) {
  const formData = new FormData();
  formData.append("file", file);
  formData.append("classId", String(extra.classId ?? 0));
  const { data } = await apiClient.post<ApiResponse<{ file: UploadedFileAsset }>>(
    `/fileUploadAndDownload/upload?noSave=${extra.noSave ?? "0"}`,
    formData,
    {
      headers: {
        "Content-Type": "multipart/form-data",
      },
    },
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "上传文件失败");
  }
  return data.data.file;
}

export async function emailTestApi(payload: {
  body?: string;
  subject?: string;
  to?: string;
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

export async function getEmailPresetListApi(params: Record<string, unknown> = {}) {
  const { data } = await apiClient.get<ApiResponse<PageResult<EmailPresetRecord>>>(
    "/email/getEmailPresetList",
    { params },
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "获取邮件预设失败");
  }
  return data.data;
}

export async function saveEmailPresetApi(payload: {
  ID?: number;
  name: string;
  description?: string;
  to: string;
  subject: string;
  body: string;
}) {
  const { data } = await apiClient.post<ApiResponse<EmailPresetRecord>>(
    "/email/saveEmailPreset",
    payload,
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "保存邮件预设失败");
  }
  return data.data;
}

export async function deleteEmailPresetApi(ID: number) {
  const { data } = await apiClient.delete<ApiResponse<Record<string, boolean>>>(
    "/email/deleteEmailPreset",
    { params: { ID } },
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "删除邮件预设失败");
  }
  return data.data;
}

export async function sendEmailApi(payload: {
  body: string;
  subject: string;
  to: string;
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
  attachments: { name: string; url: string }[];
  content: string;
  ID?: number;
  title: string;
  userID: number;
}) {
  const { data } = await apiClient.post<ApiResponse<AnnouncementRecord>>("/info/createInfo", payload);
  if (data.code !== 0) {
    throw new Error(data.msg || "创建公告失败");
  }
  return data.data;
}

export async function updateAnnouncementApi(payload: {
  attachments: { name: string; url: string }[];
  content: string;
  ID?: number;
  title: string;
  userID: number;
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

type AuthoritySaveFallbackMode = "create" | "update";
type AuthoritySaveResponsePayload = AuthorityInfo | { authority: AuthorityInfo };

function unwrapAuthoritySaveResponse(payload: AuthoritySaveResponsePayload) {
  return "authority" in payload ? payload.authority : payload;
}

function shouldFallbackAuthoritySave(error: unknown) {
  const status = Number((error as { response?: { status?: number } }).response?.status ?? 0);
  return status === 404 || status === 405;
}

const AUTHORITY_USERS_ENDPOINTS = [
  "/authority/getUsersByAuthority",
  "/api/authority/getUsersByAuthority",
] as const;

const AUTHORITY_SET_USERS_ENDPOINTS = [
  "/authority/setRoleUsers",
  "/api/authority/setRoleUsers",
] as const;

function includesAuthority(user: UserInfo, authorityId: number) {
  if (user.authorityId === authorityId) {
    return true;
  }
  if (Array.isArray(user.authorityIds) && user.authorityIds.includes(authorityId)) {
    return true;
  }
  return Array.isArray(user.authorities)
    ? user.authorities.some((authority) => authority.authorityId === authorityId)
    : false;
}

async function inferUsersByAuthorityFromUserList(authorityId: number) {
  const pageSize = 200;
  let page = 1;
  const userIds = new Set<number>();

  while (page <= 50) {
    const users = await getUserListApi({ page, pageSize });
    const list = Array.isArray(users.List) ? users.List : [];
    for (const user of list) {
      if (includesAuthority(user, authorityId)) {
        userIds.add(user.ID);
      }
    }
    const total = Number(users.Total ?? 0);
    if (list.length < pageSize || (total > 0 && page * pageSize >= total)) {
      break;
    }
    page += 1;
  }

  return Array.from(userIds);
}

async function saveAuthorityWithGinCompatEndpoint(
  payload: AuthorityUpsertInput,
  fallbackMode: AuthoritySaveFallbackMode,
) {
  const endpoint =
    fallbackMode === "update" ? "/authority/updateAuthority" : "/authority/createAuthority";
  const method = fallbackMode === "update" ? "put" : "post";
  const { data } = await apiClient.request<ApiResponse<AuthoritySaveResponsePayload>>({
    url: endpoint,
    method,
    data: payload,
  });
  if (data.code !== 0) {
    throw new Error(data.msg || "保存角色失败");
  }
  return unwrapAuthoritySaveResponse(data.data);
}

export async function saveAuthorityApi(
  payload: AuthorityUpsertInput,
  options: { fallbackMode?: AuthoritySaveFallbackMode } = {},
) {
  try {
    const { data } = await apiClient.post<ApiResponse<AuthoritySaveResponsePayload>>(
      "/authority/saveAuthority",
      payload,
    );
    if (data.code !== 0) {
      throw new Error(data.msg || "保存角色失败");
    }
    return unwrapAuthoritySaveResponse(data.data);
  } catch (error) {
    if (!shouldFallbackAuthoritySave(error)) {
      throw error;
    }
  }
  const fallbackMode = options.fallbackMode ?? (payload.ID ? "update" : "create");
  return saveAuthorityWithGinCompatEndpoint(payload, fallbackMode);
}

export async function deleteAuthorityApi(authorityId: number) {
  const { data } = await apiClient.post<ApiResponse<MutationAck>>(
    "/authority/deleteAuthority",
    { authorityId },
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "删除角色失败");
  }
  return data.data;
}

export async function getUsersByAuthorityIdApi(authorityId: number) {
  let fallbackError: unknown;
  for (const endpoint of AUTHORITY_USERS_ENDPOINTS) {
    try {
      const { data } = await apiClient.get<ApiResponse<number[]>>(endpoint, {
        params: { authorityId },
      });
      if (data.code !== 0) {
        throw new Error(data.msg || "获取角色关联用户失败");
      }
      return data.data;
    } catch (error) {
      if (!shouldFallbackAuthoritySave(error)) {
        throw error;
      }
      fallbackError = error;
    }
  }
  try {
    return inferUsersByAuthorityFromUserList(authorityId);
  } catch (error) {
    if (fallbackError) {
      throw fallbackError;
    }
    throw error;
  }
}

export async function setRoleUsersApi(payload: AuthorityRoleUsersInput) {
  let fallbackError: unknown;
  for (const endpoint of AUTHORITY_SET_USERS_ENDPOINTS) {
    try {
      const { data } = await apiClient.post<ApiResponse<MutationAck>>(endpoint, payload);
      if (data.code !== 0) {
        throw new Error(data.msg || "保存角色关联用户失败");
      }
      return data.data;
    } catch (error) {
      if (!shouldFallbackAuthoritySave(error)) {
        throw error;
      }
      fallbackError = error;
    }
  }
  if (fallbackError) {
    throw fallbackError;
  }
  throw new Error("保存角色关联用户失败");
}

export async function copyAuthorityApi(payload: AuthorityCopyInput) {
  const { data } = await apiClient.post<ApiResponse<AuthorityCopyResult>>(
    "/authority/copyAuthority",
    payload,
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "复制角色失败");
  }
  return data.data.authority;
}

export async function getAuthorityBtnApi(payload: AuthorityButtonMatrixInput) {
  const { data } = await apiClient.post<ApiResponse<AuthorityButtonMatrixSelection>>(
    "/authorityBtn/getAuthorityBtn",
    payload,
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "获取按钮权限失败");
  }
  return data.data;
}

export async function getAuthorityBtnsApi(payload: AuthorityButtonMatrixBatchInput) {
  const { data } = await apiClient.post<ApiResponse<AuthorityButtonMatrixBatchSelectionItem[]>>(
    "/authorityBtn/getAuthorityBtns",
    payload,
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "批量获取按钮权限失败");
  }
  return data.data;
}

export async function setAuthorityBtnApi(payload: AuthorityButtonMatrixInput) {
  const { data } = await apiClient.post<ApiResponse<MutationAck>>(
    "/authorityBtn/setAuthorityBtn",
    payload,
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "保存按钮权限失败");
  }
  return data.data;
}

export async function canRemoveAuthorityBtnApi(id: number) {
  const { data } = await apiClient.post<ApiResponse<{ removable?: boolean }>>(
    "/authorityBtn/canRemoveAuthorityBtn",
    undefined,
    { params: { id } },
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "校验按钮是否可删除失败");
  }
  return data.data?.removable == null ? { removable: true } : data.data;
}

export async function getMenuTreeApi() {
  const { data } = await apiClient.post<ApiResponse<MenuTreeResult>>("/menu/getMenu", {});
  if (data.code !== 0) {
    throw new Error(data.msg || "获取菜单树失败");
  }
  return data.data.menus;
}

export async function getMenuListApi() {
  const { data } = await apiClient.post<ApiResponse<PageResult<MenuInfo>>>("/menu/getMenuList", {});
  if (data.code !== 0) {
    throw new Error(data.msg || "获取菜单列表失败");
  }
  return data.data;
}

export async function saveMenuApi(payload: MenuUpsertInput) {
  const { data } = await apiClient.post<ApiResponse<MenuInfo>>("/menu/saveMenu", payload);
  if (data.code !== 0) {
    throw new Error(data.msg || "保存菜单失败");
  }
  return data.data;
}

export async function saveMenusApi(payload: MenuUpsertInput[]) {
  const { data } = await apiClient.post<ApiResponse<MenuInfo[]>>("/menu/saveMenus", {
    menus: payload,
  });
  if (data.code !== 0) {
    throw new Error(data.msg || "批量保存菜单失败");
  }
  return data.data;
}

export async function deleteBaseMenuApi(payload: IdInput) {
  const { data } = await apiClient.post<ApiResponse<MutationAck>>(
    "/menu/deleteBaseMenu",
    payload,
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "删除菜单失败");
  }
  return data.data;
}

export async function getBaseMenuByIdApi(ID: number) {
  const { data } = await apiClient.post<ApiResponse<{ menu: MenuInfo }>>(
    "/menu/getBaseMenuById",
    { ID },
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "获取菜单详情失败");
  }
  return data.data.menu;
}

export async function addMenuAuthorityApi(payload: MenuAuthorityAssignmentInput) {
  const { data } = await apiClient.post<ApiResponse<MutationAck>>(
    "/menu/addMenuAuthority",
    payload,
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "保存菜单角色关联失败");
  }
  return data.data;
}

export async function getMenuAuthorityApi(authorityId: number) {
  const { data } = await apiClient.post<ApiResponse<{ menus: MenuInfo[] }>>(
    "/menu/getMenuAuthority",
    { authorityId },
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "获取角色菜单关联失败");
  }
  return data.data.menus;
}

export async function getMenuRolesApi(menuId: number) {
  const { data } = await apiClient.get<ApiResponse<MenuRoleIdsResult>>(
    "/menu/getMenuRoles",
    { params: { menuId } },
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "获取菜单关联角色失败");
  }
  return data.data;
}

export async function setMenuRolesApi(payload: MenuRoleUpdateInput) {
  const { data } = await apiClient.post<ApiResponse<MutationAck>>(
    "/menu/setMenuRoles",
    payload,
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "保存菜单关联角色失败");
  }
  return data.data;
}

export async function getApiListApi(filters: ApiSearchInput = {}) {
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
  const endpoint = payload.ID ? "/api/updateApi" : "/api/createApi";
  const { data } = await apiClient.post<ApiResponse<ApiInfo | Record<string, never>>>(
    endpoint,
    payload,
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "保存接口失败");
  }
  return {
    ...payload,
    ...(typeof data.data === "object" && data.data ? data.data : {}),
  } as ApiInfo;
}

export async function getApiByIdApi(ID: number) {
  const { data } = await apiClient.post<ApiResponse<{ api: ApiInfo }>>("/api/getApiById", {
    id: ID,
  });
  if (data.code !== 0) {
    throw new Error(data.msg || "获取接口详情失败");
  }
  return data.data.api;
}

export async function deleteApiApi(payload: ApiDeleteInput) {
  const { data } = await apiClient.post<ApiResponse<MutationAck>>("/api/deleteApi", payload);
  if (data.code !== 0) {
    throw new Error(data.msg || "删除接口失败");
  }
  return data.data;
}

export async function deleteApisByIdsApi(payload: ApiDeleteIdsInput) {
  const { data } = await apiClient.delete<ApiResponse<MutationAck>>("/api/deleteApisByIds", {
    data: payload,
  });
  if (data.code !== 0) {
    throw new Error(data.msg || "批量删除接口失败");
  }
  return data.data;
}

export async function syncApiApi() {
  const { data } = await apiClient.get<ApiResponse<ApiSyncPreviewResult>>("/api/syncApi");
  if (data.code !== 0) {
    throw new Error(data.msg || "同步接口预览失败");
  }
  return data.data;
}

export async function getApiGroupsApi() {
  const { data } = await apiClient.get<ApiResponse<ApiGroupResult>>("/api/getApiGroups");
  if (data.code !== 0) {
    throw new Error(data.msg || "获取接口分组失败");
  }
  return data.data;
}

export async function ignoreApiApi(payload: ApiIgnoreRecord) {
  const { data } = await apiClient.post<ApiResponse<MutationAck>>("/api/ignoreApi", payload);
  if (data.code !== 0) {
    throw new Error(data.msg || "忽略接口失败");
  }
  return data.data;
}

export async function enterSyncApiApi(payload: ApiSyncCommitInput) {
  const { data } = await apiClient.post<ApiResponse<MutationAck>>("/api/enterSyncApi", payload);
  if (data.code !== 0) {
    throw new Error(data.msg || "确认同步接口失败");
  }
  return data.data;
}

export async function freshCasbinApi() {
  const { data } = await apiClient.get<ApiResponse<MutationAck>>("/api/freshCasbin");
  if (data.code !== 0) {
    throw new Error(data.msg || "刷新 Casbin 缓存失败");
  }
  return data.data;
}

export async function getApiRolesApi(payload: ApiRoleBindingQuery) {
  const { data } = await apiClient.get<ApiResponse<number[]>>("/api/getApiRoles", {
    params: payload,
  });
  if (data.code !== 0) {
    throw new Error(data.msg || "获取接口关联角色失败");
  }
  return data.data;
}

export async function setApiRolesApi(payload: ApiRoleBindingInput) {
  const { data } = await apiClient.post<ApiResponse<MutationAck>>("/api/setApiRoles", payload);
  if (data.code !== 0) {
    throw new Error(data.msg || "设置接口关联角色失败");
  }
  return data.data;
}

export async function getDictionaryListApi(filters: DictionarySearchInput = {}) {
  const { data } = await apiClient.get<ApiResponse<PageResult<DictionaryInfo>>>(
    "/sysDictionary/getSysDictionaryList",
    { params: { page: 1, pageSize: 50, ...filters } },
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "获取字典列表失败");
  }
  return data.data;
}

export async function saveDictionaryApi(payload: DictionaryUpsertInput) {
  const endpoint = payload.ID ? "/sysDictionary/updateSysDictionary" : "/sysDictionary/createSysDictionary";
  const method = payload.ID ? "put" : "post";
  const { data } = await apiClient.request<ApiResponse<DictionaryInfo | Record<string, never>>>({
    url: endpoint,
    method,
    data: payload,
  });
  if (data.code !== 0) {
    throw new Error(data.msg || "保存字典失败");
  }
  return {
    ...payload,
    ...(typeof data.data === "object" && data.data ? data.data : {}),
  } as DictionaryInfo;
}

export async function findDictionaryApi(payload: DictionarySearchInput & Partial<DictionaryDeleteInput>) {
  const { data } = await apiClient.get<ApiResponse<{ resysDictionary: DictionaryInfo }>>(
    "/sysDictionary/findSysDictionary",
    { params: payload },
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "获取字典详情失败");
  }
  return data.data.resysDictionary;
}

export async function deleteDictionaryApi(payload: DictionaryDeleteInput) {
  const { data } = await apiClient.delete<ApiResponse<MutationAck>>(
    "/sysDictionary/deleteSysDictionary",
    { data: payload },
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "删除字典失败");
  }
  return data.data;
}

export async function deleteDictionaryByIdsApi(_payload: DictionaryDeleteIdsInput) {
  const ids = _payload.ids ?? _payload.IDs ?? [];
  const { data } = await apiClient.delete<ApiResponse<MutationAck>>(
    "/sysDictionary/deleteSysDictionaryByIds",
    { data: { ids } },
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "批量删除字典失败");
  }
  return data.data;
}

export async function exportDictionaryApi(ID: number) {
  const { data } = await apiClient.get<ApiResponse<DictionaryExportRecord>>(
    "/sysDictionary/exportSysDictionary",
    { params: { ID } },
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "导出字典失败");
  }
  return data.data;
}

export async function importDictionaryApi(payload: DictionaryImportInput) {
  const { data } = await apiClient.post<ApiResponse<MutationAck>>(
    "/sysDictionary/importSysDictionary",
    payload,
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "导入字典失败");
  }
  return data.data;
}

export async function getDictionaryDetailTreeApi(sysDictionaryID: number) {
  const { data } = await apiClient.get<ApiResponse<DictionaryDetailInfo[] | { list: DictionaryDetailInfo[] }>>(
    "/sysDictionaryDetail/getDictionaryTreeList",
    {
      params: { sysDictionaryID },
    },
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "获取字典详情失败");
  }
  if (Array.isArray(data.data)) {
    return data.data;
  }
  return data.data.list;
}

export async function getParamsListApi(filters: ParamSearchInput = {}) {
  const { data } = await apiClient.get<ApiResponse<PageResult<ParamInfo>>>(
    "/sysParams/getSysParamsList",
    { params: { page: 1, pageSize: 50, ...filters } },
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "获取参数列表失败");
  }
  return data.data;
}

export async function saveParamApi(payload: ParamUpsertInput) {
  const requestPayload = {
    ...payload,
    name: payload.name?.trim() || payload.key.trim(),
  };
  const endpoint = payload.ID ? "/sysParams/updateSysParams" : "/sysParams/createSysParams";
  const method = payload.ID ? "put" : "post";
  const { data } = await apiClient.request<ApiResponse<ParamInfo | Record<string, never>>>({
    url: endpoint,
    method,
    data: requestPayload,
  });
  if (data.code !== 0) {
    throw new Error(data.msg || "保存参数失败");
  }
  return {
    ...requestPayload,
    ...(typeof data.data === "object" && data.data ? data.data : {}),
  } as ParamInfo;
}

export async function findParamApi(ID: number) {
  const { data } = await apiClient.get<ApiResponse<ParamInfo>>("/sysParams/findSysParams", {
    params: { ID },
  });
  if (data.code !== 0) {
    throw new Error(data.msg || "获取参数详情失败");
  }
  return data.data;
}

export async function deleteParamApi(payload: ParamDeleteInput) {
  const { data } = await apiClient.delete<ApiResponse<MutationAck>>("/sysParams/deleteSysParams", {
    params: { ID: payload.ID },
  });
  if (data.code !== 0) {
    throw new Error(data.msg || "删除参数失败");
  }
  return data.data;
}

export async function deleteParamsByIdsApi(payload: ParamDeleteIdsInput) {
  const IDs = payload.IDs ?? payload["IDs[]"] ?? payload.ids ?? [];
  const { data } = await apiClient.delete<ApiResponse<MutationAck>>(
    "/sysParams/deleteSysParamsByIds",
    { params: { "IDs[]": IDs } },
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "批量删除参数失败");
  }
  return data.data;
}

export async function getSysParamValueApi(payload: ParamValueQuery) {
  const { data } = await apiClient.get<ApiResponse<ParamInfo>>("/sysParams/getSysParam", {
    params: payload,
  });
  if (data.code !== 0) {
    throw new Error(data.msg || "获取参数值失败");
  }
  return data.data;
}

export async function getParamHistoryApi(_payload: ParamHistoryQuery): Promise<ParamHistoryRecord[]> {
  throw new Error("后端尚未实现 getParamHistoryApi 合同，当前仅保留前端 stub。");
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

export async function deleteUserApi(payload: UserDeleteInput) {
  const { data } = await apiClient.delete<ApiResponse<MutationAck>>(
    "/user/deleteUser",
    { data: payload },
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "删除用户失败");
  }
  return data.data;
}

export async function resetPasswordApi(payload: UserResetPasswordInput) {
  const { data } = await apiClient.post<ApiResponse<MutationAck>>(
    "/user/resetPassword",
    payload,
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "重置密码失败");
  }
  return data.data;
}

export async function setUserAuthoritiesApi(payload: UserAuthorityAssignmentInput) {
  const { data } = await apiClient.post<ApiResponse<MutationAck>>(
    "/user/setUserAuthorities",
    payload,
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "设置用户角色失败");
  }
  return data.data;
}

export async function saveDictionaryDetailApi(payload: DictionaryDetailUpsertInput) {
  const endpoint = payload.ID
    ? "/sysDictionaryDetail/updateSysDictionaryDetail"
    : "/sysDictionaryDetail/createSysDictionaryDetail";
  const method = payload.ID ? "put" : "post";
  const { data } = await apiClient.request<ApiResponse<DictionaryDetailInfo | Record<string, never>>>({
    url: endpoint,
    method,
    data: payload,
  });
  if (data.code !== 0) {
    throw new Error(data.msg || "保存字典详情失败");
  }
  return {
    ...payload,
    ...(typeof data.data === "object" && data.data ? data.data : {}),
  } as DictionaryDetailInfo;
}

export async function getDictionaryDetailApi(ID: number) {
  const { data } = await apiClient.get<ApiResponse<{ reSysDictionaryDetail: DictionaryDetailInfo }>>(
    "/sysDictionaryDetail/findSysDictionaryDetail",
    { params: { ID } },
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "获取字典节点详情失败");
  }
  return data.data.reSysDictionaryDetail;
}

export async function getDictionaryDetailListApi(filters: DictionaryDetailSearchInput = {}) {
  const { data } = await apiClient.get<ApiResponse<PageResult<DictionaryDetailInfo>>>(
    "/sysDictionaryDetail/getSysDictionaryDetailList",
    { params: { page: 1, pageSize: 50, ...filters } },
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "获取字典节点列表失败");
  }
  return data.data;
}

export async function getDictionaryTreeByTypeApi(payload: DictionaryTreeByTypeQuery) {
  const { data } = await apiClient.get<ApiResponse<{ list: DictionaryDetailInfo[] }>>(
    "/sysDictionaryDetail/getDictionaryTreeListByType",
    { params: payload },
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "按类型获取字典树失败");
  }
  return data.data.list;
}

export async function getDictionaryDetailsByParentApi(payload: DictionaryDetailsByParentInput) {
  const { data } = await apiClient.get<ApiResponse<{ list: DictionaryDetailInfo[] }>>(
    "/sysDictionaryDetail/getDictionaryDetailsByParent",
    { params: payload },
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "获取父级字典节点失败");
  }
  return data.data.list;
}

export async function getDictionaryPathApi(ID: number) {
  const { data } = await apiClient.get<ApiResponse<DictionaryPathResult>>(
    "/sysDictionaryDetail/getDictionaryPath",
    { params: { ID } },
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "获取字典节点路径失败");
  }
  return data.data;
}

export async function deleteDictionaryDetailApi(payload: DictionaryDetailDeleteInput) {
  const { data } = await apiClient.delete<ApiResponse<MutationAck>>(
    "/sysDictionaryDetail/deleteSysDictionaryDetail",
    { data: payload },
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "删除字典节点失败");
  }
  return data.data;
}

export async function deleteDictionaryDetailByIdsApi(_payload: DictionaryDetailDeleteIdsInput) {
  const ids = _payload.ids ?? _payload.IDs ?? [];
  const { data } = await apiClient.delete<ApiResponse<MutationAck>>(
    "/sysDictionaryDetail/deleteSysDictionaryDetailByIds",
    { data: { ids } },
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "批量删除字典节点失败");
  }
  return data.data;
}

export async function reorderDictionaryDetailsApi(_payload: DictionaryDetailReorderInput) {
  const { data } = await apiClient.put<ApiResponse<MutationAck>>(
    "/sysDictionaryDetail/reorderDictionaryDetails",
    _payload,
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "重排字典节点失败");
  }
  return data.data;
}

export async function getOperationLogsApi(filters: OperationLogSearchInput = {}) {
  const { data } = await apiClient.get<ApiResponse<PageResult<OperationLogInfo>>>(
    "/sysOperationRecord/getSysOperationRecordList",
    { params: { page: 1, pageSize: 50, ...filters } },
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "获取操作日志失败");
  }
  return data.data;
}

export async function getOperationLogDetailApi(ID: number) {
  const { data } = await apiClient.get<ApiResponse<{ reSysOperationRecord: OperationLogInfo }>>(
    "/sysOperationRecord/findSysOperationRecord",
    { params: { ID } },
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "获取操作日志详情失败");
  }
  return data.data.reSysOperationRecord;
}

export async function deleteOperationLogApi(payload: OperationLogDeleteInput) {
  const { data } = await apiClient.delete<ApiResponse<MutationAck>>(
    "/sysOperationRecord/deleteSysOperationRecord",
    { data: payload },
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "删除操作日志失败");
  }
  return data.data;
}

export async function deleteOperationLogsByIdsApi(payload: OperationLogDeleteIdsInput) {
  const { data } = await apiClient.delete<ApiResponse<MutationAck>>(
    "/sysOperationRecord/deleteSysOperationRecordByIds",
    { data: payload },
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "批量删除操作日志失败");
  }
  return data.data;
}

export async function exportOperationLogsApi(_payload: OperationLogExportQuery): Promise<BlobArtifact> {
  throw new Error("后端尚未实现 exportSysOperationRecordApi 合同，当前仅保留前端 stub。");
}

export async function getLoginLogsApi(filters: LoginLogSearchInput = {}) {
  const { data } = await apiClient.get<ApiResponse<PageResult<LoginLogInfo>>>(
    "/sysLoginLog/getLoginLogList",
    { params: { page: 1, pageSize: 50, ...filters } },
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "获取登录日志失败");
  }
  return data.data;
}

export async function getLoginLogDetailApi(ID: number) {
  const { data } = await apiClient.get<ApiResponse<LoginLogInfo>>("/sysLoginLog/findLoginLog", {
    params: { ID },
  });
  if (data.code !== 0) {
    throw new Error(data.msg || "获取登录日志详情失败");
  }
  return data.data;
}

export async function deleteLoginLogApi(payload: LoginLogDeleteInput) {
  const { data } = await apiClient.delete<ApiResponse<MutationAck>>("/sysLoginLog/deleteLoginLog", {
    data: payload,
  });
  if (data.code !== 0) {
    throw new Error(data.msg || "删除登录日志失败");
  }
  return data.data;
}

export async function deleteLoginLogsByIdsApi(payload: LoginLogDeleteIdsInput) {
  const { data } = await apiClient.delete<ApiResponse<MutationAck>>(
    "/sysLoginLog/deleteLoginLogByIds",
    { data: payload },
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "批量删除登录日志失败");
  }
  return data.data;
}

export async function exportLoginLogsApi(_payload: LoginLogExportQuery): Promise<BlobArtifact> {
  throw new Error("后端尚未实现 exportLoginLogApi 合同，当前仅保留前端 stub。");
}

export async function getErrorLogsApi(filters: {
  createdAfter?: string;
  createdBefore?: string;
  keyword?: string;
  path?: string;
  status?: string;
} = {}) {
  const { data } = await apiClient.get<ApiResponse<PageResult<Record<string, unknown>>>>(
    "/sysError/getSysErrorList",
    {
      params: {
        "createdAtRange[]":
          filters.createdAfter && filters.createdBefore
            ? [filters.createdAfter, filters.createdBefore]
            : undefined,
        page: 1,
        pageSize: 50,
        form: filters.path?.trim() || undefined,
        info: filters.keyword?.trim() || undefined,
        status: filters.status?.trim() || undefined,
      },
    },
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "获取错误日志失败");
  }
  return {
    ...data.data,
    List: (data.data.List ?? []).map((item) => {
      const row = item as Record<string, unknown>;
      const form = String(row.form ?? "");
      const info = String(row.info ?? "");
      return {
        ID: Number(row.ID ?? row.id ?? 0),
        createdAt: (row.CreatedAt ?? row.createdAt ?? "") as number | string,
        updatedAt: (row.UpdatedAt ?? row.updatedAt ?? "") as number | string,
        error: info,
        form,
        level: String(row.level ?? ""),
        path: form,
        solution: String(row.solution ?? ""),
        status: String(row.status ?? ""),
      } satisfies ErrorLogInfo;
    }),
  };
}

export async function deleteErrorLogApi(ID: number) {
  const { data } = await apiClient.delete<ApiResponse<MutationAck>>("/sysError/deleteSysError", {
    params: { ID },
  });
  if (data.code !== 0) {
    throw new Error(data.msg || "删除错误日志失败");
  }
  return data.data;
}

export async function deleteErrorLogsApi(ids: number[]) {
  const { data } = await apiClient.delete<ApiResponse<MutationAck>>("/sysError/deleteSysErrorByIds", {
    params: { "IDs[]": ids },
  });
  if (data.code !== 0) {
    throw new Error(data.msg || "批量删除错误日志失败");
  }
  return data.data;
}

export async function updateErrorReviewApi(payload: { ID: number; status: string; solution?: string }) {
  const { data } = await apiClient.put<ApiResponse<MutationAck>>("/sysError/updateSysError", payload);
  if (data.code !== 0) {
    throw new Error(data.msg || "更新错误日志失败");
  }
  return data.data;
}

export async function getErrorSolutionApi(ID: number) {
  const { data } = await apiClient.get<ApiResponse<MutationAck>>("/sysError/getSysErrorSolution", {
    params: { id: ID },
  });
  if (data.code !== 0) {
    throw new Error(data.msg || "触发错误方案失败");
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
  const { data } = await apiClient.post<ApiResponse<SystemConfigInfo | { config: SystemConfigInfo }>>(
    "/system/getSystemConfig",
    {},
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "获取系统配置失败");
  }
  if (data.data && typeof data.data === "object" && "config" in data.data) {
    return data.data.config;
  }
  return data.data as SystemConfigInfo;
}

export async function updateSystemConfigApi(payload: SystemConfigInfo) {
  const { data } = await apiClient.post<ApiResponse<SystemConfigInfo | { config?: SystemConfigInfo }>>(
    "/system/setSystemConfig",
    payload,
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "保存系统配置失败");
  }
  if (data.data && typeof data.data === "object" && "config" in data.data && data.data.config) {
    return data.data.config;
  }
  if (data.data && typeof data.data === "object") {
    return data.data as SystemConfigInfo;
  }
  return payload;
}

export async function reloadSystemApi(payload: ReloadSystemInput = {}) {
  const { data } = await apiClient.post<ApiResponse<MutationAck>>(
    "/system/reloadSystem",
    payload,
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "重载系统失败");
  }
  return data.data;
}

export async function submitAiModerationDecision(payload: {
  action: string;
  articleId: string;
  delegation?: {
    operatorUserId: number;
    scope: string;
  };
  mode: "delegated" | "system";
  reason: string;
  serviceToken?: string;
}) {
  const { data } = await apiClient.post<
    ApiResponse<{ accepted: boolean; auditId: string }>
  >("/ai/moderation/decision", payload);
  if (data.code !== 0) {
    throw new Error(data.msg || "提交 AI 处置失败");
  }
  return data.data;
}

export async function getAIWorkflowSessionListApi(payload: AiWorkflowSessionSearchInput = {}) {
  const { data } = await apiClient.post<ApiResponse<PageResult<AiWorkflowSessionListItem>>>(
    "/autoCode/getAIWorkflowSessionList",
    {
      page: 1,
      pageSize: 20,
      ...payload,
    },
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "获取 AI 工作流会话列表失败");
  }
  return data.data;
}

export async function getAIWorkflowSessionDetailApi(ID: number) {
  const { data } = await apiClient.post<ApiResponse<{ session: AiWorkflowSessionRecord }>>(
    "/autoCode/getAIWorkflowSessionDetail",
    { id: ID },
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "获取 AI 工作流会话详情失败");
  }
  return data.data.session;
}

export async function saveAIWorkflowSessionApi(payload: AiWorkflowSessionUpsertInput) {
  const { data } = await apiClient.post<ApiResponse<{ session: AiWorkflowSessionRecord }>>(
    "/autoCode/saveAIWorkflowSession",
    payload,
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "保存 AI 工作流会话失败");
  }
  return data.data.session;
}

export async function deleteAIWorkflowSessionApi(payload: IdInput) {
  const { data } = await apiClient.post<ApiResponse<MutationAck>>(
    "/autoCode/deleteAIWorkflowSession",
    { id: payload.ID },
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "删除 AI 工作流会话失败");
  }
  return data.data;
}

export async function dumpAIWorkflowMarkdownApi(payload: AiWorkflowSessionUpsertInput) {
  const { data } = await apiClient.post<ApiResponse<{ result: AiWorkflowMarkdownDumpResult }>>(
    "/autoCode/dumpAIWorkflowMarkdown",
    payload,
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "导出 AI 工作流 Markdown 失败");
  }
  return data.data.result;
}

export async function rollbackAIWorkflowNodeApi(payload: AiWorkflowNodeRollbackInput) {
  const { data } = await apiClient.post<ApiResponse<MutationAck>>(
    "/autoCode/rollbackAIWorkflowNode",
    payload,
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "回滚 AI 工作流节点失败");
  }
  return data.data;
}

export async function resumeAIWorkflowChatApi(payload: AiWorkflowResumeChatInput) {
  const { data } = await apiClient.post<ApiResponse<AiWorkflowSessionRecord>>(
    "/autoCode/resumeAIWorkflowChat",
    payload,
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "恢复 AI 工作流续聊失败");
  }
  return data.data;
}

export async function updateAIWorkflowStageApi(payload: AiWorkflowStageUpdateInput) {
  const { data } = await apiClient.post<ApiResponse<AiWorkflowSessionRecord>>(
    "/autoCode/updateAIWorkflowStage",
    payload,
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "更新 AI 工作流阶段失败");
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

export async function issueApiTokenApi(payload: ApiTokenIssueInput) {
  const { data } = await apiClient.post<ApiResponse<ApiTokenRecord>>(
    "/tool/api-token/issue",
    payload,
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "登记令牌失败");
  }
  return data.data;
}

export async function getApiTokenDetailApi(ID: number) {
  const { data } = await apiClient.get<ApiResponse<ApiTokenDetailRecord>>(
    "/tool/api-token/detail",
    { params: { ID } },
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "获取令牌详情失败");
  }
  return data.data;
}

export async function invalidateApiTokenApi(payload: IdInput & { reason?: string }) {
  const { data } = await apiClient.post<ApiResponse<MutationAck>>(
    "/tool/api-token/invalidate",
    payload,
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "作废令牌失败");
  }
  return data.data;
}

export async function getApiTokenCurlPayloadApi(ID: number) {
  const { data } = await apiClient.get<ApiResponse<ApiTokenCurlPayload>>(
    "/tool/api-token/curl-payload",
    { params: { ID } },
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "获取令牌 Curl 载荷失败");
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
  return data.data;
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
  kind: string;
  name: string;
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
  apiIds: number[];
  dictionaryIds: number[];
  menuGroup: string;
  menuIds: number[];
  pluginName: string;
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
  manifest: string;
  target: string;
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
  customerLevel?: string;
  customerName: string;
  customerPhoneData: string;
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
  customerLevel?: string;
  customerName: string;
  customerPhoneData: string;
  customerStatus?: string;
  ID: number;
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
  description: string;
  name: string;
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
  args: Record<string, unknown>;
  name: string;
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
  agent: string;
  allowedTools: string;
  context: string;
  description: string;
  enabled: boolean;
  markdown: string;
  name: string;
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

function assetEndpoint(kind: "reference" | "resource" | "script" | "template") {
  switch (kind) {
    case "reference": {
      return {
        create: "/skills/createReference",
        get: "/skills/getReference",
        save: "/skills/saveReference",
      };
    }
    case "resource": {
      return {
        create: "/skills/createResource",
        get: "/skills/getResource",
        save: "/skills/saveResource",
      };
    }
    case "script": {
      return {
        create: "/skills/createScript",
        get: "/skills/getScript",
        save: "/skills/saveScript",
      };
    }
    default: {
      return {
        create: "/skills/createTemplate",
        get: "/skills/getTemplate",
        save: "/skills/saveTemplate",
      };
    }
  }
}

export async function createSkillAssetApi(
  kind: "reference" | "resource" | "script" | "template",
  payload: { name: string; skillName: string; },
) {
  const endpoint = assetEndpoint(kind).create;
  const { data } = await apiClient.post<ApiResponse<{ file: SkillAssetRecord }>>(endpoint, payload);
  if (data.code !== 0) {
    throw new Error(data.msg || "创建文件失败");
  }
  return data.data.file;
}

export async function getSkillAssetApi(
  kind: "reference" | "resource" | "script" | "template",
  payload: { name: string; skillName: string; },
) {
  const endpoint = assetEndpoint(kind).get;
  const { data } = await apiClient.post<ApiResponse<{ file: SkillAssetRecord }>>(endpoint, payload);
  if (data.code !== 0) {
    throw new Error(data.msg || "获取文件失败");
  }
  return data.data.file;
}

export async function saveSkillAssetApi(
  kind: "reference" | "resource" | "script" | "template",
  payload: { content: string; name: string; skillName: string; },
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

export async function getLlmConfigListApi() {
  const { data } = await apiClient.get<ApiResponse<PageResult<LlmConfigInfo>>>(
    "/tool/llm-config/list",
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "获取大模型配置失败");
  }
  return data.data;
}

export async function saveLlmConfigApi(payload: Partial<LlmConfigInfo>) {
  const { data } = await apiClient.post<ApiResponse<LlmConfigInfo>>(
    "/tool/llm-config/save",
    payload,
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "保存大模型配置失败");
  }
  return data.data;
}

export async function deleteLlmConfigApi(ID: number) {
  const { data } = await apiClient.post<ApiResponse<Record<string, never>>>(
    "/tool/llm-config/delete",
    { ID },
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "删除大模型配置失败");
  }
}

export async function getFrontendNavListApi() {
  const { data } = await apiClient.get<ApiResponse<PageResult<FrontendNavInfo>>>(
    "/frontend/nav/list",
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "获取前端导航失败");
  }
  return data.data;
}

export async function saveFrontendNavApi(payload: Partial<FrontendNavInfo>) {
  const { data } = await apiClient.post<ApiResponse<FrontendNavInfo>>(
    "/frontend/nav/save",
    payload,
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "保存前端导航失败");
  }
  return data.data;
}

export async function deleteFrontendNavApi(ID: number) {
  const { data } = await apiClient.post<ApiResponse<Record<string, never>>>(
    "/frontend/nav/delete",
    { ID },
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "删除前端导航失败");
  }
}

export async function getArticleCategoryListApi() {
  const { data } = await apiClient.get<ApiResponse<PageResult<ArticleCategoryInfo>>>(
    "/frontend/article-category/list",
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "获取文章分类失败");
  }
  return data.data;
}

export async function saveArticleCategoryApi(payload: Partial<ArticleCategoryInfo>) {
  const { data } = await apiClient.post<ApiResponse<ArticleCategoryInfo>>(
    "/frontend/article-category/save",
    payload,
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "保存文章分类失败");
  }
  return data.data;
}

export async function deleteArticleCategoryApi(ID: number) {
  const { data } = await apiClient.post<ApiResponse<Record<string, never>>>(
    "/frontend/article-category/delete",
    { ID },
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "删除文章分类失败");
  }
}

export async function getArticleListApi() {
  const { data } = await apiClient.get<ApiResponse<PageResult<ArticleInfo>>>(
    "/frontend/article/list",
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "获取文章列表失败");
  }
  return data.data;
}

export async function saveArticleApi(payload: Partial<ArticleInfo>) {
  const { data } = await apiClient.post<ApiResponse<ArticleInfo>>(
    "/frontend/article/save",
    payload,
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "保存文章失败");
  }
  return data.data;
}

export async function deleteArticleApi(ID: number) {
  const { data } = await apiClient.post<ApiResponse<Record<string, never>>>(
    "/frontend/article/delete",
    { ID },
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "删除文章失败");
  }
}

export async function getMemberListApi() {
  const { data } = await apiClient.get<ApiResponse<PageResult<MemberInfo>>>(
    "/frontend/member/list",
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "获取会员列表失败");
  }
  return data.data;
}

export async function saveMemberApi(payload: Partial<MemberInfo>) {
  const { data } = await apiClient.post<ApiResponse<MemberInfo>>(
    "/frontend/member/save",
    payload,
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "保存会员失败");
  }
  return data.data;
}

export async function deleteMemberApi(ID: number) {
  const { data } = await apiClient.post<ApiResponse<Record<string, never>>>(
    "/frontend/member/delete",
    { ID },
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "删除会员失败");
  }
}

export async function getConsoleMenuListApi() {
  const { data } = await apiClient.get<ApiResponse<PageResult<ConsoleMenuInfo>>>(
    "/frontend/console-menu/list",
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "获取控制台菜单失败");
  }
  return data.data;
}

export async function saveConsoleMenuApi(payload: Partial<ConsoleMenuInfo>) {
  const { data } = await apiClient.post<ApiResponse<ConsoleMenuInfo>>(
    "/frontend/console-menu/save",
    payload,
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "保存控制台菜单失败");
  }
  return data.data;
}

export async function deleteConsoleMenuApi(ID: number) {
  const { data } = await apiClient.post<ApiResponse<Record<string, never>>>(
    "/frontend/console-menu/delete",
    { ID },
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "删除控制台菜单失败");
  }
}

export async function getFrontendSettingsApi() {
  const { data } = await apiClient.get<ApiResponse<FrontendSettingsInfo>>(
    "/frontend/settings/get",
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "获取前台设置失败");
  }
  return data.data;
}

export async function saveFrontendSettingsApi(payload: FrontendSettingsInfo) {
  const { data } = await apiClient.post<ApiResponse<FrontendSettingsInfo>>(
    "/frontend/settings/save",
    payload,
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "保存前台设置失败");
  }
  return data.data;
}

export async function setPolicyPathByAuthorityIdApi(payload: {
  authorityId: number;
  policies: Array<{ method: string; path: string }>;
}) {
  const { data } = await apiClient.post<ApiResponse<Record<string, never>>>(
    "/casbin/setPolicyPathByAuthorityId",
    payload,
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "保存角色 API 权限失败");
  }
}
