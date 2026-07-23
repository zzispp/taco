import '../../global.css';

import { defaultDocumentLanguage } from 'src/shared/i18n/locale-contract';

type RootEntryLayoutProps = Readonly<{
  children: React.ReactNode;
}>;

export default function RootEntryLayout({ children }: RootEntryLayoutProps) {
  return (
    <html lang={defaultDocumentLanguage} dir="ltr">
      <body>{children}</body>
    </html>
  );
}
