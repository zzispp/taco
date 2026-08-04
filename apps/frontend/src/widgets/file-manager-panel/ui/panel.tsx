'use client';

import Alert from '@mui/material/Alert';

import { paths } from 'src/shared/routes/paths';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { AdminBreadcrumbs } from 'src/shared/ui/admin-common';
import { DashboardContent } from 'src/shared/ui/dashboard-content';

import {
  fileManagerRouteKey,
  useFileManagerRoute,
  type FileManagerRoute,
  useFileManagerController,
} from 'src/features/file-management';

import { FileManagerToolbar } from './toolbar';
import { FileManagerDialogs } from './dialogs';
import { FileDetailsDrawer } from './details-drawer';
import { FileEntryCollection } from './entry-collection';

export function FileManagerPanel() {
  const route = useFileManagerRoute();
  return <FileManagerPanelContent key={fileManagerRouteKey(route)} route={route} />;
}

function FileManagerPanelContent({ route }: { route: FileManagerRoute }) {
  const { t } = useTranslate('admin');
  const controller = useFileManagerController(route);
  return (
    <DashboardContent maxWidth="xl">
      <AdminBreadcrumbs
        heading={t('file.managerTitle')}
        parentLinks={[{ name: t('nav.fileManagement'), href: paths.dashboard.file }]}
        onRefresh={controller.actions.refreshPage}
        refreshing={
          controller.resources.entries.isValidating ||
          controller.resources.overview.isValidating ||
          controller.resources.spaceSelector.spaces.isValidating ||
          controller.resources.directoryTrailValidating
        }
      />
      {!controller.resources.spaceId && controller.resources.overview.error ? (
        <Alert severity="error" sx={{ mb: 2 }}>
          {t('file.noSpace')}
        </Alert>
      ) : null}
      <FileManagerToolbar controller={controller} />
      <FileEntryCollection controller={controller} />
      <FileDetailsDrawer controller={controller} />
      <FileManagerDialogs controller={controller} />
    </DashboardContent>
  );
}
