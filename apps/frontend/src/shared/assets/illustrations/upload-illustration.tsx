import type { SvgIconProps } from '@mui/material/SvgIcon';

import { memo } from 'react';

import SvgIcon from '@mui/material/SvgIcon';

import { BackgroundShape } from './background-shape';
import { UploadIllustrationPart1 } from './upload-illustration-part-1';
import { UploadIllustrationPart2 } from './upload-illustration-part-2';
import { UploadIllustrationPart3 } from './upload-illustration-part-3';
import { UploadIllustrationPart4 } from './upload-illustration-part-4';
import { UploadIllustrationPart5 } from './upload-illustration-part-5';

// ----------------------------------------------------------------------

type SvgProps = SvgIconProps & { hideBackground?: boolean };

function UploadIllustration({ hideBackground, sx, ...other }: SvgProps) {
  return (
    <SvgIcon
      viewBox="0 0 480 360"
      xmlns="http://www.w3.org/2000/svg"
      sx={[
        (theme) => ({
          '--primary-main': theme.vars.palette.primary.main,
          '--primary-dark': theme.vars.palette.primary.dark,
          '--primary-darker': theme.vars.palette.primary.darker,
          width: 320,
          maxWidth: 1,
          flexShrink: 0,
          height: 'auto',
        }),
        ...(Array.isArray(sx) ? sx : [sx]),
      ]}
      {...other}
    >
      {!hideBackground && <BackgroundShape />}

      <UploadIllustrationPart1 />
      <UploadIllustrationPart2 />
      <UploadIllustrationPart3 />
      <UploadIllustrationPart4 />
      <UploadIllustrationPart5 />
    </SvgIcon>
  );
}

export default memo(UploadIllustration);
