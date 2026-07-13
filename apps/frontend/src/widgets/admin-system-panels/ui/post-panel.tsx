'use client';

import type { Post, PostInput } from 'src/entities/system';

import { useMemo } from 'react';

import { useTable } from 'src/shared/ui/table';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { useLocalDateTimeFilterState } from 'src/shared/lib/use-local-date-time-filter-state';

import { usePosts } from 'src/entities/system';

import { systemMutations } from 'src/features/system-management';

import { SystemCrudPanel } from 'src/widgets/system-crud-panel';

import { PostToolbar } from './post-toolbar';
import { postFields, toPostFilters, postFilterFields } from './post-fields';
import { DEFAULT_POST_INPUT, DEFAULT_POST_FILTERS } from './post-constants';

export function PostManagementPanel() {
  const { t } = useTranslate('admin');
  const table = useTable({ defaultRowsPerPage: 10 });
  const filters = useLocalDateTimeFilterState(DEFAULT_POST_FILTERS, {
    onValidQuery: table.onResetPage,
  });
  const resource = usePosts(table.page, table.rowsPerPage, filters.query);
  const fields = useMemo(() => postFields(t), [t]);
  const filterFields = useMemo(() => postFilterFields(t), [t]);

  return (
    <SystemCrudPanel<Post, PostInput>
      title={t('pages.postManagement')}
      addLabel={t('actions.addPost')}
      idKey="post_id"
      nameKey="post_name"
      fields={fields}
      defaultInput={DEFAULT_POST_INPUT}
      resource={resource}
      page={table.page}
      rowsPerPage={table.rowsPerPage}
      filters={filterFields}
      filterValues={filters.draft}
      filterError={filters.error}
      permissionPrefix="system:post"
      onFilterChange={(next) => filters.change(toPostFilters(next))}
      onPageChange={table.onChangePage}
      onRowsPerPageChange={table.onChangeRowsPerPage}
      createItem={systemMutations.createPost}
      updateItem={systemMutations.updatePost}
      deleteItem={systemMutations.deletePost}
      batchDeleteItems={systemMutations.deletePosts}
      toolbarAction={<PostToolbar filters={filters.query} filterError={filters.error} />}
    />
  );
}
