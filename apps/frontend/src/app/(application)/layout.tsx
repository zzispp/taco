import { ApplicationProviders } from 'src/app/providers';

type ApplicationLayoutProps = Readonly<{
  children: React.ReactNode;
}>;

export default function ApplicationLayout({ children }: ApplicationLayoutProps) {
  return <ApplicationProviders>{children}</ApplicationProviders>;
}
