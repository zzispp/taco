import type { Menu } from 'src/entities/menu';
import type { NavSectionProps } from 'src/shared/ui/nav-section';

import useSWR from 'swr';
import { useMemo } from 'react';

import { fetcher } from 'src/shared/api/http-client';

const NAVBAR_ENDPOINT = '/api/navbar';

export type BackendNavSection = {
  code: string;
  subheader: string;
  items: BackendNavItem[];
};

export type BackendNavItem = {
  code: string;
  title: string;
  path: string;
  icon: string | null;
  caption: string | null;
  deep_match: boolean;
  children: BackendNavItem[];
};

export type NavResponse = {
  nav_items: BackendNavSection[];
};

const swrOptions = {
  keepPreviousData: true,
  revalidateOnFocus: false,
};

export function useNavbar() {
  const { data, isLoading, error, isValidating } = useSWR<NavResponse>(
    NAVBAR_ENDPOINT,
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

export function mapMenuItemToNav(item: Menu): NavSectionProps['data'][number]['items'][number] {
  return {
    code: item.menu_id,
    title: item.menu_name,
    path: item.path,
    icon: item.icon ?? undefined,
    deepMatch: true,
  };
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
