import { mutate } from 'swr';

import { isEndpointKey } from 'src/shared/api/pagination';

import { fileEndpoints, isFileDirectoryTrailKey } from 'src/entities/file';

export async function refreshFileResources() {
  await mutate((key) => isEndpointKey(key, fileEndpoints.assets));
  await mutate(isFileDirectoryTrailKey);
  await mutate(isFileOverviewKey);
  await mutate((key) => isEndpointKey(key, fileEndpoints.spaces));
  await mutate((key) => isEndpointKey(key, fileEndpoints.providers));
}

function isFileOverviewKey(key: unknown) {
  const endpoint = fileEndpoints.overview();
  return typeof key === 'string' && (key === endpoint || key.startsWith(`${endpoint}?`));
}
