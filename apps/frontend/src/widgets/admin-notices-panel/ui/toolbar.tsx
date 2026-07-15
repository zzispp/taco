import type { NoticeManagementController } from 'src/features/notice-management';

import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';

import { Iconify } from 'src/shared/ui/iconify';
import { useTranslate } from 'src/shared/i18n/use-locales';

import { NOTICE_TYPE, noticeTypeTranslationKeys } from 'src/entities/notice';

import { AddButton } from 'src/widgets/admin-common';

export function NoticeToolbar({ controller }: { controller: NoticeManagementController }) {
  return (
    <Box sx={{ mb: 2, p: 2, border: 1, borderColor: 'divider', borderRadius: 1 }}>
      <Stack direction={{ xs: 'column', md: 'row' }} spacing={1.5} useFlexGap flexWrap="wrap">
        <NoticeFilters controller={controller} />
        <NoticeToolbarActions controller={controller} />
      </Stack>
    </Box>
  );
}

function NoticeFilters({ controller }: { controller: NoticeManagementController }) {
  const { t } = useTranslate('admin');
  const { state } = controller;
  return (
    <>
      <TextField
        size="small"
        label={t('notice.fields.title')}
        value={state.filterDraft.notice_title}
        sx={{ minWidth: 200 }}
        onChange={(event) =>
          state.setFilterDraft({ ...state.filterDraft, notice_title: event.target.value })
        }
      />
      <TextField
        size="small"
        label={t('notice.fields.createBy')}
        value={state.filterDraft.create_by}
        sx={{ minWidth: 160 }}
        onChange={(event) =>
          state.setFilterDraft({ ...state.filterDraft, create_by: event.target.value })
        }
      />
      <NoticeTypeFilter controller={controller} />
    </>
  );
}

function NoticeTypeFilter({ controller }: { controller: NoticeManagementController }) {
  const { t } = useTranslate('admin');
  const { state } = controller;
  return (
    <TextField
      select
      size="small"
      label={t('notice.fields.type')}
      value={state.filterDraft.notice_type}
      sx={{ minWidth: 140 }}
      onChange={(event) =>
        state.setFilterDraft({
          ...state.filterDraft,
          notice_type: event.target.value as typeof state.filterDraft.notice_type,
        })
      }
    >
      <MenuItem value="">{t('common.all')}</MenuItem>
      {Object.values(NOTICE_TYPE).map((type) => (
        <MenuItem key={type} value={type}>
          {t(noticeTypeTranslationKeys[type])}
        </MenuItem>
      ))}
    </TextField>
  );
}

function NoticeToolbarActions({ controller }: { controller: NoticeManagementController }) {
  const { t } = useTranslate('admin');
  const { state, permissions, actions } = controller;
  return (
    <>
      <Button
        variant="outlined"
        startIcon={<Iconify icon="eva:search-fill" />}
        onClick={actions.applyFilters}
      >
        {t('common.search')}
      </Button>
      <Button
        variant="outlined"
        startIcon={<Iconify icon="solar:restart-bold" />}
        onClick={actions.resetFilters}
      >
        {t('common.reset')}
      </Button>
      <Box sx={{ flex: 1 }} />
      {permissions.canRemove ? (
        <Button
          color="error"
          variant="outlined"
          disabled={!state.table.selected.length}
          onClick={() => state.setBatchDeleteOpen(true)}
        >
          {t('common.delete')}
        </Button>
      ) : null}
      {permissions.canAdd ? (
        <AddButton onClick={() => state.setCreating(true)}>{t('actions.addNotice')}</AddButton>
      ) : null}
    </>
  );
}
