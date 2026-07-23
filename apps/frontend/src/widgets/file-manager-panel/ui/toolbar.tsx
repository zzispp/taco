import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import TextField from '@mui/material/TextField';
import ToggleButton from '@mui/material/ToggleButton';
import ToggleButtonGroup from '@mui/material/ToggleButtonGroup';

import { Iconify } from 'src/shared/ui/iconify';
import { useTranslate } from 'src/shared/i18n/use-locales';

import {
  type FileViewMode,
  FileSpaceSelector,
  type FileManagerMode,
  canSelectFileEntries,
  canUseFileBatchAction,
  type FileManagerController,
} from 'src/features/file-management';

import { FileManagerDirectoryNavigation } from './directory-navigation';

export function FileManagerToolbar({ controller }: { controller: FileManagerController }) {
  return (
    <Box sx={{ mb: 3, p: 2, border: 1, borderColor: 'divider', borderRadius: 1 }}>
      <Stack spacing={2}>
        <ModeAndSpace controller={controller} />
        <FileManagerDirectoryNavigation controller={controller} />
        <FilterRow controller={controller} />
      </Stack>
    </Box>
  );
}

function ModeAndSpace({ controller }: { controller: FileManagerController }) {
  const { t } = useTranslate('admin');
  const { state, resources } = controller;
  return (
    <Stack direction={{ xs: 'column', md: 'row' }} spacing={1.5} alignItems={{ md: 'center' }}>
      <ToggleButtonGroup
        exclusive
        size="small"
        value={state.mode}
        onChange={(_, value: FileManagerMode | null) => value && state.setMode(value)}
      >
        <ToggleButton value="active">{t('file.modes.active')}</ToggleButton>
        <ToggleButton value="trash">{t('file.modes.trash')}</ToggleButton>
      </ToggleButtonGroup>
      {controller.permissions.canListSpaces ? (
        <FileSpaceSelector
          selector={resources.spaceSelector}
          currentUserId={resources.currentUserId}
          label={t('file.fields.space')}
          onChange={state.setSpaceId}
        />
      ) : null}
      <Box sx={{ flex: 1 }} />
      <ViewSwitch controller={controller} />
    </Stack>
  );
}

function FilterRow({ controller }: { controller: FileManagerController }) {
  const { t } = useTranslate('admin');
  const { state, actions } = controller;
  return (
    <Stack direction={{ xs: 'column', md: 'row' }} spacing={1.5} useFlexGap flexWrap="wrap">
      <TextField
        size="small"
        label={t('file.filters.search')}
        value={state.filterDraft.search}
        onChange={(event) =>
          state.setFilterDraft({ ...state.filterDraft, search: event.target.value })
        }
      />
      <TextField
        size="small"
        label={t('file.filters.extension')}
        value={state.filterDraft.extension}
        onChange={(event) =>
          state.setFilterDraft({ ...state.filterDraft, extension: event.target.value })
        }
      />
      <TextField
        size="small"
        label={t('file.filters.tag')}
        value={state.filterDraft.tag}
        onChange={(event) =>
          state.setFilterDraft({ ...state.filterDraft, tag: event.target.value })
        }
      />
      <Button
        variant="outlined"
        startIcon={<Iconify icon="eva:search-fill" />}
        onClick={actions.applyFilters}
      >
        {t('file.actions.apply')}
      </Button>
      <Button
        variant="outlined"
        startIcon={<Iconify icon="solar:restart-bold" />}
        onClick={actions.resetFilters}
      >
        {t('file.actions.reset')}
      </Button>
      <Box sx={{ flex: 1 }} />
      <BatchActions controller={controller} />
      <CreationActions controller={controller} />
    </Stack>
  );
}

function BatchActions({ controller }: { controller: FileManagerController }) {
  const { t } = useTranslate('admin');
  const { state, permissions } = controller;
  if (!canSelectFileEntries(state.mode, permissions) || !state.table.selected.length) return null;
  const canTrash = canUseFileBatchAction(state.mode, permissions, 'trash');
  const canRestore = canUseFileBatchAction(state.mode, permissions, 'restore');
  const canPurge = canUseFileBatchAction(state.mode, permissions, 'purge');
  return (
    <Stack direction="row" spacing={1} useFlexGap flexWrap="wrap">
      {state.mode === 'trash' ? (
        <>
          {canRestore ? (
            <Button
              variant="outlined"
              startIcon={<Iconify icon="solar:restart-bold" />}
              onClick={() => controller.actions.requestBatchAction('restore')}
            >
              {t('file.actions.restoreSelected')}
            </Button>
          ) : null}
          {canPurge ? (
            <Button
              color="error"
              variant="outlined"
              startIcon={<Iconify icon="solar:trash-bin-trash-bold" />}
              onClick={() => controller.actions.requestBatchAction('purge')}
            >
              {t('file.actions.purgeSelected')}
            </Button>
          ) : null}
        </>
      ) : canTrash ? (
        <Button
          color="error"
          variant="outlined"
          startIcon={<Iconify icon="solar:trash-bin-trash-bold" />}
          onClick={() => controller.actions.requestBatchAction('trash')}
        >
          {t('file.actions.trashSelected')}
        </Button>
      ) : null}
    </Stack>
  );
}

function CreationActions({ controller }: { controller: FileManagerController }) {
  const { t } = useTranslate('admin');
  if (controller.state.mode === 'trash') return null;
  return (
    <Stack direction="row" spacing={1}>
      {controller.permissions.canAddFolder ? (
        <Button
          variant="outlined"
          startIcon={<Iconify icon="solar:add-folder-bold" />}
          onClick={controller.actions.openFolderDialog}
        >
          {t('file.actions.newFolder')}
        </Button>
      ) : null}
      {controller.permissions.canUpload ? (
        <Button
          variant="contained"
          startIcon={<Iconify icon="eva:cloud-upload-fill" />}
          onClick={controller.actions.openUpload}
        >
          {t('file.actions.upload')}
        </Button>
      ) : null}
    </Stack>
  );
}

function ViewSwitch({ controller }: { controller: FileManagerController }) {
  const { t } = useTranslate('admin');
  return (
    <ToggleButtonGroup
      exclusive
      size="small"
      value={controller.state.viewMode}
      onChange={(_, value: FileViewMode | null) => value && controller.state.setViewMode(value)}
    >
      <ToggleButton value="list" aria-label={t('file.views.list')}>
        <Iconify icon="solar:list-bold" />
      </ToggleButton>
      <ToggleButton value="grid" aria-label={t('file.views.grid')}>
        <Iconify icon="mingcute:dot-grid-fill" />
      </ToggleButton>
    </ToggleButtonGroup>
  );
}
