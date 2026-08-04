'use client';

import type { FileThumbnailProps } from './types';

import { mergeClasses } from 'minimal-shared/utils';

import Tooltip from '@mui/material/Tooltip';

import { Iconify } from '../iconify';
import { fileThumbnailClasses } from './classes';
import { getFileMeta, getFileIcon } from './utils';
import { useFilePreview } from './use-file-preview';
import { RemoveButton, ThumbnailRoot, DownloadButton, ThumbnailImage } from './styles';

// ----------------------------------------------------------------------

export function FileThumbnail({
  sx,
  file,
  tooltip,
  onRemove,
  showImage,
  slotProps,
  className,
  onDownload,
  previewUrl: previewUrlProp,
  ...other
}: FileThumbnailProps) {
  const fileMeta = getFileMeta(file);

  const hasPreviewUrlProp = previewUrlProp !== undefined;
  const previewEnabled = !hasPreviewUrlProp && !!file && !!showImage && fileMeta.format === 'image';
  const { previewUrl } = useFilePreview(previewEnabled ? file : null);

  const imageSrc = hasPreviewUrlProp ? previewUrlProp : previewUrl;
  const canShowImage = Boolean(fileMeta.format === 'image' && showImage && imageSrc);

  const content = (
    <ThumbnailRoot
      className={mergeClasses([fileThumbnailClasses.root, className])}
      sx={sx}
      {...other}
    >
      <ThumbnailMedia
        canShowImage={canShowImage}
        fileMeta={fileMeta}
        imageSrc={imageSrc}
        slotProps={slotProps}
      />
      <ThumbnailActions onRemove={onRemove} onDownload={onDownload} slotProps={slotProps} />
    </ThumbnailRoot>
  );

  if (!file) return null;

  return tooltip ? (
    <FileThumbnailTooltip
      content={content}
      name={fileMeta.name}
      tooltipProps={slotProps?.tooltip}
    />
  ) : (
    content
  );
}

type FileThumbnailTooltipProps = {
  name: string;
  content: React.ReactElement;
  tooltipProps?: NonNullable<FileThumbnailProps['slotProps']>['tooltip'];
};

function FileThumbnailTooltip({ content, name, tooltipProps }: FileThumbnailTooltipProps) {
  return (
    <Tooltip
      arrow
      title={name}
      {...tooltipProps}
      slotProps={{
        ...tooltipProps?.slotProps,
        popper: {
          modifiers: [{ name: 'offset', options: { offset: [0, -12] } }],
          ...tooltipProps?.slotProps?.popper,
        },
      }}
    >
      {content}
    </Tooltip>
  );
}

type ThumbnailMediaProps = {
  canShowImage: boolean;
  imageSrc?: string;
  fileMeta: ReturnType<typeof getFileMeta>;
  slotProps?: FileThumbnailProps['slotProps'];
};

function ThumbnailMedia({ canShowImage, fileMeta, imageSrc, slotProps }: ThumbnailMediaProps) {
  const imageProps = canShowImage ? slotProps?.img : slotProps?.icon;

  return (
    <ThumbnailImage
      {...(canShowImage && { showImage: true })}
      alt={fileMeta.name}
      src={canShowImage ? imageSrc : getFileIcon(fileMeta.format)}
      className={canShowImage ? fileThumbnailClasses.img : fileThumbnailClasses.icon}
      {...imageProps}
    />
  );
}

type ThumbnailActionsProps = Pick<FileThumbnailProps, 'onDownload' | 'onRemove' | 'slotProps'>;

function ThumbnailActions({ onRemove, onDownload, slotProps }: ThumbnailActionsProps) {
  return (
    <>
      {onRemove && (
        <RemoveButton
          onClick={onRemove}
          className={fileThumbnailClasses.removeBtn}
          {...slotProps?.removeBtn}
        >
          <Iconify icon="mingcute:close-line" width={12} />
        </RemoveButton>
      )}
      {onDownload && (
        <DownloadButton
          onClick={onDownload}
          className={fileThumbnailClasses.downloadBtn}
          {...slotProps?.downloadBtn}
        >
          <Iconify width={24} icon="eva:cloud-download-fill" />
        </DownloadButton>
      )}
    </>
  );
}
