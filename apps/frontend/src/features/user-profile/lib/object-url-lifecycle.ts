export type ObjectUrlApi = Readonly<{
  createObjectURL: (value: Blob) => string;
  revokeObjectURL: (url: string) => void;
}>;

export type ObjectUrlLifecycle = Readonly<{
  replace: (value: Blob) => string;
  clear: () => void;
}>;

export function createObjectUrlLifecycle(api: ObjectUrlApi): ObjectUrlLifecycle {
  let currentUrl: string | null = null;
  return {
    replace(value) {
      revokeCurrentUrl();
      currentUrl = api.createObjectURL(value);
      return currentUrl;
    },
    clear: revokeCurrentUrl,
  };

  function revokeCurrentUrl() {
    if (!currentUrl) return;
    api.revokeObjectURL(currentUrl);
    currentUrl = null;
  }
}
