'use client';

import type { AvatarProps } from '@mui/material/Avatar';
import type { FileEntry } from '../model/types';
import type { FileThumbnailProps } from 'src/shared/ui/file-thumbnail';

import Badge from '@mui/material/Badge';
import Avatar from '@mui/material/Avatar';
import Tooltip from '@mui/material/Tooltip';

import { FileThumbnail } from 'src/shared/ui/file-thumbnail';

import { supportsFileThumbnail } from '../model/media';
import { useFileThumbnailObjectUrl } from '../model/use-file-thumbnail-object-url';

type ManagedMediaProps = Readonly<{
  entry: FileEntry;
  enabled?: boolean;
  errorLabel: string;
}>;

type ManagedFileThumbnailProps = ManagedMediaProps &
  Omit<FileThumbnailProps, 'file' | 'previewUrl' | 'showImage'>;

export function ManagedFileThumbnail({
  entry,
  enabled = true,
  errorLabel,
  ...props
}: ManagedFileThumbnailProps) {
  const canLoad = enabled && supportsFileThumbnail(entry);
  const media = useFileThumbnailObjectUrl(entry.id, canLoad);
  const thumbnail = (
    <FileThumbnail
      {...props}
      file={entry.type === 'folder' ? 'folder' : entry.name}
      previewUrl={canLoad ? media.url : undefined}
      showImage={canLoad && Boolean(media.url)}
    />
  );
  return renderMediaStatus(thumbnail, media.error, errorLabel);
}

type ManagedFileImageProps = ManagedMediaProps & Omit<AvatarProps, 'src' | 'alt'>;

export function ManagedFileImage({
  entry,
  enabled = true,
  errorLabel,
  ...props
}: ManagedFileImageProps) {
  const canLoad = enabled && supportsFileThumbnail(entry);
  const media = useFileThumbnailObjectUrl(entry.id, canLoad);
  const image = <Avatar {...props} src={media.url || undefined} alt={entry.name} />;
  return renderMediaStatus(image, media.error, errorLabel);
}

function renderMediaStatus(content: React.ReactElement, error: unknown | null, errorLabel: string) {
  if (!error) return content;
  return (
    <Tooltip title={errorLabel}>
      <Badge color="error" variant="dot" overlap="rectangular">
        {content}
      </Badge>
    </Tooltip>
  );
}
