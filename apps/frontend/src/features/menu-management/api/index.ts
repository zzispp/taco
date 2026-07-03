import type { MenuItem, MenuSection, MenuItemInput, MenuSectionInput } from 'src/entities/menu/model/types';

import { mutate } from 'swr';

import axios from 'src/shared/api/http-client';
import { requestData, isEndpointKey } from 'src/shared/api/pagination';

import { menuEndpoints } from 'src/entities/menu/api/endpoints';

const NAVBAR_ENDPOINT = '/api/navbar';

export async function createMenuSection(payload: MenuSectionInput) {
  const section = await requestData<MenuSection>(axios.post(menuEndpoints.menuSections, payload));
  await mutate((key) => isEndpointKey(key, menuEndpoints.menuSections));
  await mutate(NAVBAR_ENDPOINT);
  return section;
}

export async function updateMenuSection(id: string, payload: MenuSectionInput) {
  const section = await requestData<MenuSection>(axios.put(menuEndpoints.menuSection(id), payload));
  await mutate((key) => isEndpointKey(key, menuEndpoints.menuSections));
  await mutate(NAVBAR_ENDPOINT);
  return section;
}

export async function deleteMenuSection(id: string) {
  await axios.delete(menuEndpoints.menuSection(id));
  await mutate((key) => isEndpointKey(key, menuEndpoints.menuSections));
  await mutate(NAVBAR_ENDPOINT);
}

export async function createMenuItem(payload: MenuItemInput) {
  const item = await requestData<MenuItem>(axios.post(menuEndpoints.menuItems, payload));
  await mutate((key) => isEndpointKey(key, menuEndpoints.menuItems));
  await mutate(NAVBAR_ENDPOINT);
  return item;
}

export async function updateMenuItem(id: string, payload: MenuItemInput) {
  const item = await requestData<MenuItem>(axios.put(menuEndpoints.menuItem(id), payload));
  await mutate((key) => isEndpointKey(key, menuEndpoints.menuItems));
  await mutate(NAVBAR_ENDPOINT);
  return item;
}

export async function deleteMenuItem(id: string) {
  await axios.delete(menuEndpoints.menuItem(id));
  await mutate((key) => isEndpointKey(key, menuEndpoints.menuItems));
  await mutate(NAVBAR_ENDPOINT);
}
