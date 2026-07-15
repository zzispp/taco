type TerminalSessionRejectionOptions = Readonly<{
  clearSession: () => Promise<void>;
  refreshAuthState: () => Promise<void>;
  redirectToSignIn: () => void;
}>;

export async function endSessionAfterTerminalRejection(
  options: TerminalSessionRejectionOptions
): Promise<void> {
  await options.clearSession();
  await options.refreshAuthState();
  options.redirectToSignIn();
}
