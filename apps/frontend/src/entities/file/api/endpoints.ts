export const fileEndpoints = {
  assets: '/api/system/files',
  overview: (spaceId?: string) =>
    `/api/system/files/overview${spaceId ? `?space_id=${encodeURIComponent(spaceId)}` : ''}`,
  spaces: '/api/system/file-spaces',
  providers: '/api/system/file-providers',
  space: (id: string) => `/api/system/file-spaces/${encodeURIComponent(id)}`,
  uploadSessions: '/api/system/file-upload-sessions',
  uploadSession: (id: string) => `/api/system/file-upload-sessions/${encodeURIComponent(id)}`,
  uploadSessionPart: (id: string, partNumber: number) =>
    `/api/system/file-upload-sessions/${encodeURIComponent(id)}/parts/${partNumber}`,
  uploadSessionComplete: (id: string) =>
    `/api/system/file-upload-sessions/${encodeURIComponent(id)}/complete`,
  folders: '/api/system/files/folders',
  asset: (id: string) => `/api/system/files/${encodeURIComponent(id)}`,
  directoryTrail: (id: string) => `/api/system/files/${encodeURIComponent(id)}/directory-trail`,
  content: (id: string) => `/api/system/files/${encodeURIComponent(id)}/content`,
  preview: (id: string) => `/api/system/files/${encodeURIComponent(id)}/preview`,
  thumbnail: (id: string) => `/api/system/files/${encodeURIComponent(id)}/thumbnail`,
  trash: (id: string) => `/api/system/files/${encodeURIComponent(id)}/trash`,
  restore: (id: string) => `/api/system/files/${encodeURIComponent(id)}/restore`,
  purge: (id: string) => `/api/system/files/${encodeURIComponent(id)}/purge`,
  batchTrash: '/api/system/files/trash/batch',
  batchRestore: '/api/system/files/trash/restore/batch',
  batchPurge: '/api/system/files/trash/purge/batch',
} as const;
