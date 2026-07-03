'use client';

import type { NavSectionProps } from 'src/components/nav-section';
import type {
  Role,
  MenuItem,
  UserInput,
  RoleInput,
  SystemUser,
  NavResponse,
  MenuSection,
  PageResponse,
  MenuItemInput,
  ApiPermission,
  BackendNavItem,
  RoleApiBinding,
  RoleMenuBinding,
  MenuSectionInput,
  BackendNavSection,
  ApiPermissionInput,
} from 'src/types/rbac';

import { useMemo } from 'react';
import useSWR, { mutate } from 'swr';

import axios, { fetcher, endpoints } from 'src/lib/axios';

// ----------------------------------------------------------------------

const swrOptions = {
  keepPreviousData: true,
  revalidateOnFocus: false,
};

export const pageQuery = (page: number, pageSize: number) => ({
  page: page + 1,
  page_size: pageSize,
});

async function requestData<T>(request: Promise<{ data: T }>) {
  const response = await request;
  return response.data;
}

function pageKey(endpoint: string, page: number, pageSize: number) {
  return [endpoint, { params: pageQuery(page, pageSize) }] as const;
}

function usePagedResource<T>(endpoint: string, page: number, pageSize: number) {
  const { data, isLoading, error, isValidating } = useSWR<PageResponse<T>>(
    pageKey(endpoint, page, pageSize),
    fetcher,
    swrOptions
  );

  return useMemo(
    () => ({
      data,
      items: data?.items ?? [],
      total: data?.total ?? 0,
      isLoading,
      error,
      isValidating,
    }),
    [data, error, isLoading, isValidating]
  );
}

export function useRoles(page: number, pageSize: number) {
  return usePagedResource<Role>(endpoints.rbac.roles, page, pageSize);
}

export function useApis(page: number, pageSize: number) {
  return usePagedResource<ApiPermission>(endpoints.rbac.apis, page, pageSize);
}

export function useMenuSections(page: number, pageSize: number) {
  return usePagedResource<MenuSection>(endpoints.rbac.menuSections, page, pageSize);
}

export function useMenuItems(page: number, pageSize: number) {
  return usePagedResource<MenuItem>(endpoints.rbac.menuItems, page, pageSize);
}

export function useUsers(page: number, pageSize: number) {
  return usePagedResource<SystemUser>(endpoints.users, page, pageSize);
}

export function useNavbar() {
  const { data, isLoading, error, isValidating } = useSWR<NavResponse>(
    endpoints.navbar,
    fetcher,
    swrOptions
  );

  return useMemo(
    () => ({
      data: toNavSections(data?.nav_items ?? []),
      isLoading,
      error,
      isValidating,
    }),
    [data, error, isLoading, isValidating]
  );
}

export async function createRole(payload: RoleInput) {
  const role = await requestData<Role>(axios.post(endpoints.rbac.roles, payload));
  await mutate((key) => isEndpointKey(key, endpoints.rbac.roles));
  await mutate(endpoints.navbar);
  return role;
}

export async function updateRole(code: string, payload: RoleInput) {
  const role = await requestData<Role>(axios.put(endpoints.rbac.role(code), payload));
  await mutate((key) => isEndpointKey(key, endpoints.rbac.roles));
  await mutate(endpoints.navbar);
  return role;
}

export async function deleteRole(code: string) {
  await axios.delete(endpoints.rbac.role(code));
  await mutate((key) => isEndpointKey(key, endpoints.rbac.roles));
  await mutate(endpoints.navbar);
}

export async function getRoleApis(code: string) {
  return requestData<RoleApiBinding>(axios.get(endpoints.rbac.roleApis(code)));
}

export async function updateRoleApis(code: string, apiPermissionIds: string[]) {
  await axios.put(endpoints.rbac.roleApis(code), { api_permission_ids: apiPermissionIds });
  await mutate(endpoints.navbar);
}

export async function getRoleMenus(code: string) {
  return requestData<RoleMenuBinding>(axios.get(endpoints.rbac.roleMenus(code)));
}

export async function updateRoleMenus(code: string, menuItemIds: string[]) {
  await axios.put(endpoints.rbac.roleMenus(code), { menu_item_ids: menuItemIds });
  await mutate(endpoints.navbar);
}

export async function createApi(payload: ApiPermissionInput) {
  const api = await requestData<ApiPermission>(axios.post(endpoints.rbac.apis, payload));
  await mutate((key) => isEndpointKey(key, endpoints.rbac.apis));
  return api;
}

export async function updateApi(id: string, payload: ApiPermissionInput) {
  const api = await requestData<ApiPermission>(axios.put(endpoints.rbac.api(id), payload));
  await mutate((key) => isEndpointKey(key, endpoints.rbac.apis));
  return api;
}

export async function deleteApi(id: string) {
  await axios.delete(endpoints.rbac.api(id));
  await mutate((key) => isEndpointKey(key, endpoints.rbac.apis));
}

export async function createMenuSection(payload: MenuSectionInput) {
  const section = await requestData<MenuSection>(axios.post(endpoints.rbac.menuSections, payload));
  await mutate((key) => isEndpointKey(key, endpoints.rbac.menuSections));
  await mutate(endpoints.navbar);
  return section;
}

export async function updateMenuSection(id: string, payload: MenuSectionInput) {
  const section = await requestData<MenuSection>(axios.put(endpoints.rbac.menuSection(id), payload));
  await mutate((key) => isEndpointKey(key, endpoints.rbac.menuSections));
  await mutate(endpoints.navbar);
  return section;
}

export async function deleteMenuSection(id: string) {
  await axios.delete(endpoints.rbac.menuSection(id));
  await mutate((key) => isEndpointKey(key, endpoints.rbac.menuSections));
  await mutate(endpoints.navbar);
}

export async function createMenuItem(payload: MenuItemInput) {
  const item = await requestData<MenuItem>(axios.post(endpoints.rbac.menuItems, payload));
  await mutate((key) => isEndpointKey(key, endpoints.rbac.menuItems));
  await mutate(endpoints.navbar);
  return item;
}

export async function updateMenuItem(id: string, payload: MenuItemInput) {
  const item = await requestData<MenuItem>(axios.put(endpoints.rbac.menuItem(id), payload));
  await mutate((key) => isEndpointKey(key, endpoints.rbac.menuItems));
  await mutate(endpoints.navbar);
  return item;
}

export async function deleteMenuItem(id: string) {
  await axios.delete(endpoints.rbac.menuItem(id));
  await mutate((key) => isEndpointKey(key, endpoints.rbac.menuItems));
  await mutate(endpoints.navbar);
}

export async function createUser(payload: UserInput) {
  const user = await requestData<SystemUser>(axios.post(endpoints.users, payload));
  await mutate((key) => isEndpointKey(key, endpoints.users));
  return user;
}

export async function updateUser(id: string, payload: UserInput) {
  const user = await requestData<SystemUser>(axios.put(endpoints.user(id), payload));
  await mutate((key) => isEndpointKey(key, endpoints.users));
  return user;
}

export async function deleteUser(id: string) {
  await axios.delete(endpoints.user(id));
  await mutate((key) => isEndpointKey(key, endpoints.users));
}

function isEndpointKey(key: unknown, endpoint: string) {
  return key === endpoint || (Array.isArray(key) && key[0] === endpoint);
}

function toNavSections(sections: BackendNavSection[]): NavSectionProps['data'] {
  return sections.map((section) => ({
    code: section.code,
    subheader: section.subheader,
    items: section.items.map(toNavItem),
  }));
}

function toNavItem(item: BackendNavItem): NavSectionProps['data'][number]['items'][number] {
  return {
    code: item.code,
    title: item.title,
    path: item.path,
    icon: item.icon ?? undefined,
    caption: item.caption ?? undefined,
    deepMatch: item.deep_match,
    children: item.children.length ? item.children.map(toNavItem) : undefined,
  };
}
