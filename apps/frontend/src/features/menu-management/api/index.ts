import type { TreeSelectNode } from 'src/entities/system';
import type { Menu, MenuInput } from 'src/entities/menu/model/types';

import { mutate } from 'swr';

import axios from 'src/shared/api/http-client';
import { requestData, isEndpointKey } from 'src/shared/api/pagination';

import { menuEndpoints } from 'src/entities/menu/api/endpoints';

const NAVBAR_ENDPOINT = '/api/navbar';

export async function createMenu(payload: MenuInput) {
  const menu = await requestData<Menu>(axios.post(menuEndpoints.menus, payload));
  await refreshMenus();
  return menu;
}

export async function updateMenu(id: string, payload: MenuInput) {
  const menu = await requestData<Menu>(axios.put(menuEndpoints.menu(id), payload));
  await refreshMenus();
  return menu;
}

export async function updateMenuSort(id: string, orderNum: number) {
  const menu = await requestData<Menu>(axios.put(menuEndpoints.sort(id), { order_num: orderNum }));
  await refreshMenus();
  return menu;
}


export function getMenuTreeSelect() {
  return requestData<TreeSelectNode[]>(axios.get(menuEndpoints.treeSelect));
}

export async function updateMenuSorts(items: { id: string; order_num: number }[]) {
  const menus = await requestData<Menu[]>(axios.put(menuEndpoints.sortBatch, { items }));
  await refreshMenus();
  return menus;
}

export async function deleteMenu(id: string) {
  await axios.delete(menuEndpoints.menu(id));
  await refreshMenus();
}

async function refreshMenus() {
  await mutate((key) => isEndpointKey(key, menuEndpoints.menus));
  await mutate(NAVBAR_ENDPOINT);
}
