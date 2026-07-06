'use client';

import type { Post, PostInput } from 'src/entities/system';

import { useMemo, useState } from 'react';

import Button from '@mui/material/Button';

import { useTable } from 'src/shared/ui/table';
import { toast } from 'src/shared/ui/snackbar';
import { Iconify } from 'src/shared/ui/iconify';
import { useTranslate } from 'src/shared/i18n/use-locales';

import { usePosts } from 'src/entities/system';
import { useHasPermission } from 'src/entities/session';

import { systemMutations } from 'src/features/system-management';

import { SystemCrudPanel } from 'src/widgets/system-crud-panel';

const DEFAULT_INPUT: PostInput = {
  post_code: '',
  post_name: '',
  post_sort: 0,
  status: '0',
  remark: '',
};
const DEFAULT_FILTERS = { post_name: '', post_code: '', status: '' };

export function PostManagementPanel() {
  const { t } = useTranslate('admin');
  const table = useTable({ defaultRowsPerPage: 10 });
  const [filters, setFilters] = useState(DEFAULT_FILTERS);
  const resource = usePosts(table.page, table.rowsPerPage, filters);
  const canExport = useHasPermission('system:post:export');
  const fields = useMemo(
    () => [
      { key: 'post_name' as const, label: t('fields.postName') },
      { key: 'post_code' as const, label: t('fields.postCode') },
      { key: 'post_sort' as const, label: t('fields.postSort'), type: 'number' as const },
      {
        key: 'status' as const,
        label: t('common.status'),
        type: 'select' as const,
        options: statusOptions(t),
      },
      { key: 'remark' as const, label: t('common.remark'), type: 'textarea' as const },
      {
        key: 'create_time' as const,
        label: t('fields.createTime'),
        format: 'dateTime' as const,
        hiddenInForm: true,
      },
    ],
    [t]
  );
  const filterFields = useMemo(
    () => [
      { key: 'post_name', label: t('fields.postName') },
      { key: 'post_code', label: t('fields.postCode') },
      {
        key: 'status',
        label: t('common.status'),
        type: 'select' as const,
        options: allStatusOptions(t),
      },
    ],
    [t]
  );

  const exportItems = async () => {
    try {
      await systemMutations.exportPosts(filters);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.exportFailed'));
    }
  };

  return (
    <SystemCrudPanel<Post, PostInput>
      title={t('pages.postManagement')}
      addLabel={t('actions.addPost')}
      idKey="post_id"
      nameKey="post_name"
      fields={fields}
      defaultInput={DEFAULT_INPUT}
      resource={resource}
      page={table.page}
      rowsPerPage={table.rowsPerPage}
      filters={filterFields}
      filterValues={filters}
      permissionPrefix="system:post"
      onFilterChange={(next) => setFilters(toPostFilters(next))}
      onPageChange={table.onChangePage}
      onRowsPerPageChange={table.onChangeRowsPerPage}
      createItem={systemMutations.createPost}
      updateItem={systemMutations.updatePost}
      deleteItem={systemMutations.deletePost}
      batchDeleteItems={systemMutations.deletePosts}
      toolbarAction={
        canExport ? (
          <Button
            variant="outlined"
            startIcon={<Iconify icon="solar:export-bold" />}
            onClick={exportItems}
          >
            {t('actions.export')}
          </Button>
        ) : null
      }
    />
  );
}

function statusOptions(t: ReturnType<typeof useTranslate>['t']) {
  return [
    { value: '0', label: t('common.enabled') },
    { value: '1', label: t('common.disabled') },
  ];
}

function allStatusOptions(t: ReturnType<typeof useTranslate>['t']) {
  return [{ value: '', label: t('common.all') }, ...statusOptions(t)];
}

function toPostFilters(values: Record<string, string>) {
  return {
    post_name: values.post_name ?? '',
    post_code: values.post_code ?? '',
    status: values.status ?? '',
  };
}
