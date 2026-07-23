'use client';

import type { FileManagerController } from 'src/features/file-management';

import Box from '@mui/material/Box';
import Alert from '@mui/material/Alert';
import Stack from '@mui/material/Stack';
import Tooltip from '@mui/material/Tooltip';
import ButtonBase from '@mui/material/ButtonBase';
import IconButton from '@mui/material/IconButton';
import Typography from '@mui/material/Typography';
import Breadcrumbs from '@mui/material/Breadcrumbs';
import CircularProgress from '@mui/material/CircularProgress';

import { Iconify } from 'src/shared/ui/iconify';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { getErrorMessage } from 'src/shared/lib/get-error-message';

import { fileDirectoryBreadcrumbs } from 'src/features/file-management';

type FileManagerDirectoryNavigationProps = Readonly<{
  controller: FileManagerController;
}>;

export function FileManagerDirectoryNavigation({
  controller,
}: FileManagerDirectoryNavigationProps) {
  const { t } = useTranslate('admin');
  const navigation = useDirectoryNavigationState(controller, t('file.root'));
  if (navigation.error) return <Alert severity="error">{getErrorMessage(navigation.error)}</Alert>;
  return (
    <Stack direction="row" spacing={1} alignItems="center" sx={{ minWidth: 0 }}>
      <UpOneLevelButton controller={controller} navigation={navigation} />
      <DirectoryBreadcrumbContent controller={controller} navigation={navigation} />
    </Stack>
  );
}

function useDirectoryNavigationState(controller: FileManagerController, rootName: string) {
  const { directoryTrail, directoryTrailError, directoryTrailLoading } = controller.resources;
  const atRoot = !controller.state.parentId;
  return {
    atRoot,
    error: directoryTrailError,
    loading: directoryTrailLoading,
    ready: atRoot || directoryTrail.length > 0,
    breadcrumbs: fileDirectoryBreadcrumbs(rootName, directoryTrail),
  };
}

type DirectoryNavigationState = ReturnType<typeof useDirectoryNavigationState>;

function UpOneLevelButton({
  controller,
  navigation,
}: Readonly<{ controller: FileManagerController; navigation: DirectoryNavigationState }>) {
  const { t } = useTranslate('admin');
  const label = t('file.actions.upOneLevel');
  const { atRoot, ready } = navigation;
  return (
    <Tooltip title={label}>
      <span>
        <IconButton
          size="small"
          aria-label={label}
          disabled={atRoot || !ready}
          onClick={controller.actions.goToParentFolder}
        >
          <Iconify icon="eva:arrow-ios-back-fill" />
        </IconButton>
      </span>
    </Tooltip>
  );
}

function DirectoryBreadcrumbContent({
  controller,
  navigation,
}: Readonly<{ controller: FileManagerController; navigation: DirectoryNavigationState }>) {
  return (
    <Box sx={{ minWidth: 0, flex: 1, overflowX: 'auto' }}>
      {navigation.ready ? (
        <DirectoryBreadcrumbs controller={controller} navigation={navigation} />
      ) : navigation.loading ? (
        <DirectoryTrailLoading />
      ) : null}
    </Box>
  );
}

function DirectoryBreadcrumbs({
  controller,
  navigation,
}: Readonly<{ controller: FileManagerController; navigation: DirectoryNavigationState }>) {
  const { t } = useTranslate('admin');
  return (
    <Breadcrumbs
      separator={<Iconify icon="eva:arrow-ios-forward-fill" width={16} />}
      aria-label={t('file.managerTitle')}
      sx={{ '& .MuiBreadcrumbs-ol': { flexWrap: 'nowrap' } }}
    >
      {navigation.breadcrumbs.map((item, index) => (
        <DirectoryBreadcrumb
          key={item.id ?? 'root'}
          item={item}
          current={index === navigation.breadcrumbs.length - 1}
          onNavigate={() => controller.actions.goToDirectory(item.directoryTrail)}
        />
      ))}
    </Breadcrumbs>
  );
}

function DirectoryBreadcrumb({
  item,
  current,
  onNavigate,
}: Readonly<{
  item: DirectoryNavigationState['breadcrumbs'][number];
  current: boolean;
  onNavigate: () => void;
}>) {
  if (current)
    return (
      <Typography variant="body2" noWrap color="text.primary">
        {item.name}
      </Typography>
    );
  return (
    <ButtonBase onClick={onNavigate} sx={{ color: 'primary.main' }}>
      <Typography variant="body2" noWrap>
        {item.name}
      </Typography>
    </ButtonBase>
  );
}

function DirectoryTrailLoading() {
  const { t } = useTranslate('admin');
  return (
    <Stack direction="row" spacing={1} alignItems="center">
      <Typography variant="body2" color="text.secondary" noWrap>
        {t('file.root')}
      </Typography>
      <CircularProgress size={16} />
    </Stack>
  );
}
