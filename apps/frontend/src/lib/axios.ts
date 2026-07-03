import type { AxiosRequestConfig } from 'axios';

import axios from 'axios';

import { CONFIG } from 'src/global-config';

// ----------------------------------------------------------------------

const axiosInstance = axios.create({
  baseURL: CONFIG.serverUrl,
  headers: {
    'Content-Type': 'application/json',
  },
});

/**
 * Optional: Add token (if using auth)
 *
 axiosInstance.interceptors.request.use((config) => {
  const token = localStorage.getItem('jwt_access_token');
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});
*
*/

axiosInstance.interceptors.response.use(
  (response) => response,
  (error) => {
    const message = error?.response?.data?.message || error?.message || 'Something went wrong!';
    console.error('Axios error:', message);
    return Promise.reject(new Error(message));
  }
);

export default axiosInstance;

// ----------------------------------------------------------------------

export const fetcher = async <T = unknown>(
  args: string | [string, AxiosRequestConfig]
): Promise<T> => {
  try {
    const [url, config] = Array.isArray(args) ? args : [args, {}];

    const res = await axiosInstance.get<T>(url, config);

    return res.data;
  } catch (error) {
    console.error('Fetcher failed:', error);
    throw error;
  }
};

// ----------------------------------------------------------------------

export const endpoints = {
  chat: '/api/chat',
  kanban: '/api/kanban',
  calendar: '/api/calendar',
  auth: {
    me: '/api/auth/me',
    refresh: '/api/auth/refresh',
    signIn: '/api/auth/sign-in',
    signUp: '/api/auth/sign-up',
  },
  navbar: '/api/navbar',
  users: '/api/users',
  user: (id: string) => `/api/users/${id}`,
  rbac: {
    roles: '/api/rbac/roles',
    role: (code: string) => `/api/rbac/roles/${code}`,
    roleApis: (code: string) => `/api/rbac/roles/${code}/apis`,
    roleMenus: (code: string) => `/api/rbac/roles/${code}/menus`,
    apis: '/api/rbac/apis',
    api: (id: string) => `/api/rbac/apis/${id}`,
    menuSections: '/api/rbac/menu-sections',
    menuSection: (id: string) => `/api/rbac/menu-sections/${id}`,
    menuItems: '/api/rbac/menu-items',
    menuItem: (id: string) => `/api/rbac/menu-items/${id}`,
  },
} as const;
