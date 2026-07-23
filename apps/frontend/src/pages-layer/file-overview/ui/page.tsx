import { LocalizedDashboardDocumentTitle } from 'src/shared/i18n';

import { FileStorageOverviewPanel } from 'src/widgets/file-storage-overview';

export function FileOverviewPage() {
  return (
    <>
      <LocalizedDashboardDocumentTitle titleKey="file.overviewTitle" />
      <FileStorageOverviewPanel />
    </>
  );
}
