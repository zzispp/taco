type ReloginOptions = Readonly<{
  clearSession: () => Promise<void>;
  refreshAuthState: () => Promise<void>;
  redirectToSignIn: () => void;
}>;

export async function reloginAfterSessionExpired(options: ReloginOptions): Promise<void> {
  await options.clearSession();
  await options.refreshAuthState();
  options.redirectToSignIn();
}
