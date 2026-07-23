import { redirect } from 'next/navigation';

import { defaultLocaleHomePath } from 'src/shared/i18n/locale-contract';

export default function RootPage() {
  redirect(defaultLocaleHomePath);
}
