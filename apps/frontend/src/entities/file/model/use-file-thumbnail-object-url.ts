'use client';

import { useRef, useState, useEffect } from 'react';

import { getFileThumbnailBlob } from '../api/content';
import { type FileObjectUrlLifecycle, createFileObjectUrlLifecycle } from './object-url-lifecycle';

type FileThumbnailResource = Readonly<{
  key: string | null;
  url: string;
  loading: boolean;
  error: unknown | null;
}>;

const IDLE_RESOURCE: FileThumbnailResource = {
  key: null,
  url: '',
  loading: false,
  error: null,
};

export function useFileThumbnailObjectUrl(fileId: string, enabled = true) {
  const lifecycle = useFileObjectUrlLifecycle();
  const [resource, setResource] = useState<FileThumbnailResource>(IDLE_RESOURCE);
  const requestKey = enabled ? fileId : null;

  useEffect(() => {
    lifecycle.clear();
    if (!requestKey) {
      setResource(IDLE_RESOURCE);
      return undefined;
    }
    const request = new AbortController();
    setResource({ key: requestKey, url: '', loading: true, error: null });
    void loadThumbnail({ fileId, requestKey, request, lifecycle, setResource });
    return () => {
      request.abort();
      lifecycle.clear();
    };
  }, [fileId, lifecycle, requestKey]);

  if (resource.key === requestKey) return resource;
  return { key: requestKey, url: '', loading: Boolean(requestKey), error: null };
}

function useFileObjectUrlLifecycle() {
  const lifecycle = useRef<FileObjectUrlLifecycle | null>(null);
  if (!lifecycle.current) lifecycle.current = createFileObjectUrlLifecycle(URL);
  return lifecycle.current;
}

type LoadThumbnailOptions = Readonly<{
  fileId: string;
  requestKey: string;
  request: AbortController;
  lifecycle: FileObjectUrlLifecycle;
  setResource: React.Dispatch<React.SetStateAction<FileThumbnailResource>>;
}>;

async function loadThumbnail(options: LoadThumbnailOptions) {
  try {
    const blob = await getFileThumbnailBlob(options.fileId, options.request.signal);
    if (options.request.signal.aborted) return;
    const url = options.lifecycle.replace(blob);
    options.setResource({ key: options.requestKey, url, loading: false, error: null });
  } catch (error) {
    if (options.request.signal.aborted) return;
    options.setResource({ key: options.requestKey, url: '', loading: false, error });
  }
}
