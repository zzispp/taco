import type { FileRejection } from 'react-dropzone';

import { varAlpha, mergeClasses } from 'minimal-shared/utils';

import { styled } from '@mui/material/styles';

import { fData } from 'src/utils/format-number';

import { uploadClasses } from '../classes';
import { getFileMeta } from '../../file-thumbnail';

// ----------------------------------------------------------------------

export type RejectedFilesProps = React.ComponentProps<typeof RejectedList> & {
  files?: readonly FileRejection[];
};

export function RejectedFiles({ files = [], sx, className, ...other }: RejectedFilesProps) {
  return (
    <RejectedList className={mergeClasses([uploadClasses.rejected, className])} sx={sx} {...other}>
      {files.map(({ file, errors }) => {
        const fileMeta = getFileMeta(file);

        return (
          <RejectedItem key={fileMeta.key}>
            <RejectedTitle>
              {fileMeta.name} - {fileMeta.size ? fData(fileMeta.size) : ''}
            </RejectedTitle>
            {errors.map((error) => (
              <RejectedMsg key={error.code}>- {error.message}</RejectedMsg>
            ))}
          </RejectedItem>
        );
      })}
    </RejectedList>
  );
}

// ----------------------------------------------------------------------

const RejectedList = styled('ul')(({ theme }) => ({
  display: 'flex',
  gap: theme.spacing(1),
  flexDirection: 'column',
  padding: theme.spacing(2),
  marginTop: theme.spacing(3),
  borderRadius: theme.shape.borderRadius,
  border: `dashed 1px ${theme.vars.palette.error.main}`,
  backgroundColor: varAlpha(theme.vars.palette.error.mainChannel, 0.08),
}));

const RejectedItem = styled('li')({
  display: 'flex',
  flexDirection: 'column',
});

const RejectedTitle = styled('span')(({ theme }) => ({
  ...theme.typography.subtitle2,
}));

const RejectedMsg = styled('span')(({ theme }) => ({
  ...theme.typography.caption,
}));
