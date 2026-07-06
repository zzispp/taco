import { LocalizedDashboardDocumentTitle } from 'src/shared/i18n';

import { AccountProfilePanel } from 'src/widgets/account-profile-panel';

export function ProfilePage() {
  return (
    <>
      <LocalizedDashboardDocumentTitle titleKey="profile.personalCenter" />
      <AccountProfilePanel />
    </>
  );
}
