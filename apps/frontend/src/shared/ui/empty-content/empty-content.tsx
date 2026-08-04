'use client';

import type { BoxProps } from '@mui/material/Box';
import type { Theme, SxProps } from '@mui/material/styles';
import type { TypographyProps } from '@mui/material/Typography';

import { varAlpha } from 'minimal-shared/utils';

import Box from '@mui/material/Box';
import { styled } from '@mui/material/styles';
import Typography from '@mui/material/Typography';

import { CONFIG } from 'src/shared/config';

// ----------------------------------------------------------------------

export type EmptyContentProps = React.ComponentProps<'div'> & {
  title?: string;
  imgUrl?: string;
  filled?: boolean;
  sx?: SxProps<Theme>;
  description?: string;
  action?: React.ReactNode;
  slotProps?: {
    img?: BoxProps<'img'>;
    title?: TypographyProps;
    description?: TypographyProps;
  };
};

type EmptyContentSlotProps = NonNullable<EmptyContentProps['slotProps']>;

export function EmptyContent({
  sx,
  imgUrl,
  action,
  filled,
  slotProps,
  description,
  title = 'No data',
  ...other
}: EmptyContentProps) {
  return (
    <ContentRoot filled={filled} sx={sx} {...other}>
      <EmptyContentImage imgUrl={imgUrl} imgProps={slotProps?.img} />
      <EmptyContentTitle title={title} titleProps={slotProps?.title} />
      <EmptyContentDescription
        description={description}
        descriptionProps={slotProps?.description}
      />
      {action && action}
    </ContentRoot>
  );
}

// ----------------------------------------------------------------------

function EmptyContentImage({
  imgUrl,
  imgProps,
}: {
  imgUrl?: string;
  imgProps?: EmptyContentSlotProps['img'];
}) {
  return (
    <Box
      component="img"
      alt="Empty content"
      src={imgUrl ?? `${CONFIG.assetsDir}/assets/icons/empty/ic-content.svg`}
      {...imgProps}
      sx={[
        { width: 1, maxWidth: 160 },
        ...(Array.isArray(imgProps?.sx) ? imgProps.sx : [imgProps?.sx]),
      ]}
    />
  );
}

function EmptyContentTitle({
  title,
  titleProps,
}: {
  title?: string;
  titleProps?: EmptyContentSlotProps['title'];
}) {
  if (!title) return null;

  return (
    <Typography
      variant="h6"
      {...titleProps}
      sx={[
        { mt: 1, textAlign: 'center', color: 'text.disabled' },
        ...(Array.isArray(titleProps?.sx) ? titleProps.sx : [titleProps?.sx]),
      ]}
    >
      {title}
    </Typography>
  );
}

function EmptyContentDescription({
  description,
  descriptionProps,
}: {
  description?: string;
  descriptionProps?: EmptyContentSlotProps['description'];
}) {
  if (!description) return null;

  return (
    <Typography
      variant="body2"
      {...descriptionProps}
      sx={[
        { mt: 1, textAlign: 'center', color: 'text.disabled' },
        ...(Array.isArray(descriptionProps?.sx) ? descriptionProps.sx : [descriptionProps?.sx]),
      ]}
    >
      {description}
    </Typography>
  );
}

// ----------------------------------------------------------------------

const ContentRoot = styled('div', {
  shouldForwardProp: (prop: string) => !['filled', 'sx'].includes(prop),
})<Pick<EmptyContentProps, 'filled'>>(({ filled, theme }) => ({
  flexGrow: 1,
  height: '100%',
  display: 'flex',
  alignItems: 'center',
  flexDirection: 'column',
  justifyContent: 'center',
  padding: theme.spacing(0, 3),
  ...(filled && {
    borderRadius: Number(theme.shape.borderRadius) * 2,
    backgroundColor: varAlpha(theme.vars.palette.grey['500Channel'], 0.04),
    border: `dashed 1px ${varAlpha(theme.vars.palette.grey['500Channel'], 0.08)}`,
  }),
}));
