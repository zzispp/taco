import { LocalizedDashboardDocumentTitle } from 'src/shared/i18n';

import { FileSpacePanel } from 'src/widgets/file-space-panel';

export function FileSpacesPage() {
  return (
    <>
      <LocalizedDashboardDocumentTitle titleKey="file.spacesTitle" />
      <FileSpacePanel />
    </>
  );
}
