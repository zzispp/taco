import { SetupRouteGate } from 'src/app/installation/setup-route-gate';

type SetupRouteLayoutProps = Readonly<{
  children: React.ReactNode;
}>;

export default function SetupRouteLayout({ children }: SetupRouteLayoutProps) {
  return <SetupRouteGate>{children}</SetupRouteGate>;
}
