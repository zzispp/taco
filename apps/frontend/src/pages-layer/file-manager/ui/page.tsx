import { LocalizedDashboardDocumentTitle } from 'src/shared/i18n';

import { FileManagerPanel } from 'src/widgets/file-manager-panel';

export function FileManagerPage() {
  return (
    <>
      <LocalizedDashboardDocumentTitle titleKey="file.managerTitle" />
      <FileManagerPanel />
    </>
  );
}
