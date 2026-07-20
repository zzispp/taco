'use client';

import { InstallationStatusGate } from './installation-status-gate';

type SetupRouteGateProps = Readonly<{
  children: React.ReactNode;
}>;

export function SetupRouteGate({ children }: SetupRouteGateProps) {
  return <InstallationStatusGate expectedState="setup">{children}</InstallationStatusGate>;
}
