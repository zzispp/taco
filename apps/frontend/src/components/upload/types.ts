import type { DropzoneOptions } from 'react-dropzone';
import type { Theme, SxProps } from '@mui/material/styles';
import type { UploadWrapper } from './default/styles';
import type { RejectedFiles } from './components/rejected-files';
import type { PreviewOrientation, MultiFilePreviewProps } from './components/multi-file-preview';

// ----------------------------------------------------------------------

export type FileUploadType = File | string | null;
export type FilesUploadType = (File | string)[];

export type UploadProps = DropzoneOptions & {
  error?: boolean;
  loading?: boolean;
  className?: string;
  sx?: SxProps<Theme>;
  hideFilesRejected?: boolean;
  helperText?: React.ReactNode;
  placeholder?: React.ReactNode;
  previewOrientation?: PreviewOrientation;
  value?: FileUploadType | FilesUploadType;
  onDelete?: () => void;
  onUpload?: () => void;
  onRemoveAll?: () => void;
  onRemove?: (file: File | string) => void;
  slotProps?: {
    wrapper?: React.ComponentProps<typeof UploadWrapper>;
    multiPreview?: Partial<MultiFilePreviewProps>;
    rejectedFiles?: React.ComponentProps<typeof RejectedFiles>;
  };
};
