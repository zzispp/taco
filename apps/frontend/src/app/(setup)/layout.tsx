import { SetupProviders } from 'src/app/providers';

type SetupLayoutProps = Readonly<{
  children: React.ReactNode;
}>;

export default function SetupLayout({ children }: SetupLayoutProps) {
  return <SetupProviders>{children}</SetupProviders>;
}
