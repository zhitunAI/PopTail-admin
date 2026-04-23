import { apiClient, clearStoredToken, setStoredToken } from "./client";
import type {
  ApiResponse,
  LoginResult,
  PolicyPath,
  UserInfo,
  UserInfoResult,
} from "#/types/gin-ai-admin";

export async function loginApi(username: string, password: string) {
  const { data } = await apiClient.post<ApiResponse<LoginResult>>("/base/login", {
    username,
    password,
  });
  if (data.code !== 0) {
    throw new Error(data.msg || "登录失败");
  }
  setStoredToken(data.data.token, String(data.data.expiresAt));
  return data.data;
}

export async function getUserInfoApi() {
  const { data } = await apiClient.get<ApiResponse<UserInfoResult>>("/user/getUserInfo");
  if (data.code !== 0) {
    throw new Error(data.msg || "获取用户信息失败");
  }
  return data.data.userInfo;
}

export async function getPolicyPathByAuthorityId(authorityId: number) {
  const { data } = await apiClient.post<ApiResponse<PolicyPath[]>>(
    "/casbin/getPolicyPathByAuthorityId",
    {
      authorityId,
    },
  );
  if (data.code !== 0) {
    throw new Error(data.msg || "获取权限列表失败");
  }
  return data.data;
}

export async function logoutApi() {
  try {
    await apiClient.post("/base/logout");
  } finally {
    clearStoredToken();
  }
}

export async function setUserAuthorityApi(authorityId: number) {
  const { data } = await apiClient.post<
    ApiResponse<{
      user: UserInfo;
      token: string;
      expiresAt: number;
    }>
  >("/user/setUserAuthority", {
    authorityId,
  });
  if (data.code !== 0) {
    throw new Error(data.msg || "切换角色失败");
  }
  setStoredToken(data.data.token, String(data.data.expiresAt));
  return data.data;
}
