export type FileObjectUrlApi = Readonly<{
  createObjectURL: (value: Blob) => string;
  revokeObjectURL: (url: string) => void;
}>;

export type FileObjectUrlLifecycle = Readonly<{
  replace: (value: Blob) => string;
  clear: () => void;
}>;

export function createFileObjectUrlLifecycle(api: FileObjectUrlApi): FileObjectUrlLifecycle {
  let currentUrl: string | null = null;

  function clear() {
    if (!currentUrl) return;
    api.revokeObjectURL(currentUrl);
    currentUrl = null;
  }

  return {
    replace(value) {
      clear();
      currentUrl = api.createObjectURL(value);
      return currentUrl;
    },
    clear,
  };
}
