import type { FileUploadType } from '../types';

import { mergeClasses } from 'minimal-shared/utils';

import { styled } from '@mui/material/styles';

import { uploadClasses } from '../classes';
import { FileThumbnail } from '../../file-thumbnail';

// ----------------------------------------------------------------------

const SINGLE_PREVIEW_ICON_SIZE = 64;

export type SingleFilePreviewProps = React.ComponentProps<typeof PreviewRoot> & {
  file: FileUploadType;
};

export function SingleFilePreview({ sx, file, className, ...other }: SingleFilePreviewProps) {
  return (
    <PreviewRoot
      className={mergeClasses([uploadClasses.preview.single, className])}
      sx={sx}
      {...other}
    >
      <FileThumbnail
        file={file}
        showImage
        sx={{ width: '100%', height: '100%' }}
        slotProps={{
          icon: { sx: { width: SINGLE_PREVIEW_ICON_SIZE, height: SINGLE_PREVIEW_ICON_SIZE } },
        }}
      />
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
