import type { FileUploadType } from '../types';

import { mergeClasses } from 'minimal-shared/utils';

import { styled } from '@mui/material/styles';

import { uploadClasses } from '../classes';
import { getFileMeta, useFilePreview } from '../../file-thumbnail';

// ----------------------------------------------------------------------

export type SingleFilePreviewProps = React.ComponentProps<typeof PreviewRoot> & {
  file: FileUploadType;
};

export function SingleFilePreview({ sx, file, className, ...other }: SingleFilePreviewProps) {
  const fileMeta = getFileMeta(file);
  const { previewUrl } = useFilePreview(file);

  return (
    <PreviewRoot
      className={mergeClasses([uploadClasses.preview.single, className])}
      sx={sx}
      {...other}
    >
      {previewUrl && <PreviewImage alt={fileMeta.name} src={previewUrl} />}
    </PreviewRoot>
  );
}

// ----------------------------------------------------------------------

const PreviewRoot = styled('div')(({ theme }) => ({
  top: 0,
  left: 0,
  width: '100%',
  height: '100%',
  position: 'absolute',
  borderRadius: 'inherit',
  padding: theme.spacing(1),
}));

const PreviewImage = styled('img')({
  width: '100%',
  height: '100%',
  objectFit: 'cover',
  borderRadius: 'inherit',
});
