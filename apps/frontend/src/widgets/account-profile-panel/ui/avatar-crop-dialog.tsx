'use client';

import Cropper from 'react-easy-crop';

import Box from '@mui/material/Box';
import Tab from '@mui/material/Tab';
import Tabs from '@mui/material/Tabs';
import Alert from '@mui/material/Alert';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Slider from '@mui/material/Slider';
import Dialog from '@mui/material/Dialog';
import Avatar from '@mui/material/Avatar';
import Tooltip from '@mui/material/Tooltip';
import IconButton from '@mui/material/IconButton';
import Typography from '@mui/material/Typography';
import DialogTitle from '@mui/material/DialogTitle';
import DialogContent from '@mui/material/DialogContent';
import DialogActions from '@mui/material/DialogActions';
import CircularProgress from '@mui/material/CircularProgress';

import { Iconify } from 'src/shared/ui/iconify';
import { useTranslate } from 'src/shared/i18n/use-locales';

import {
  type AvatarSource,
  AVATAR_SOURCE_ASSETS,
  AVATAR_SOURCE_UPLOAD,
  useAvatarCropController,
  type AvatarCropController,
} from 'src/features/user-profile';

import { AvatarAssetPicker } from './avatar-asset-picker';

const MIN_ZOOM = 1;
const MAX_ZOOM = 3;
const ZOOM_STEP = 0.1;

export type AvatarCropDialogProps = Readonly<{
  open: boolean;
  currentAvatar: string;
  onClose: () => void;
  onUploaded: () => Promise<void>;
}>;

export function AvatarCropDialog({
  open,
  currentAvatar,
  onClose,
  onUploaded,
}: AvatarCropDialogProps) {
  const { t } = useTranslate('admin');
  const controller = useAvatarCropController({ open, onClose, onUploaded, t });
  return (
    <Dialog fullWidth maxWidth="md" open={open} onClose={controller.handleClose}>
      <DialogTitle>{t('profile.changeAvatar')}</DialogTitle>
      <DialogContent>
        <AvatarSourceTabs controller={controller} t={t} />
        {controller.source === AVATAR_SOURCE_UPLOAD ? (
          <AvatarCropBody controller={controller} currentAvatar={currentAvatar} t={t} />
        ) : (
          <AssetSourceBody controller={controller} currentAvatar={currentAvatar} t={t} />
        )}
      </DialogContent>
      <AvatarDialogActions controller={controller} t={t} />
    </Dialog>
  );
}

type Translate = ReturnType<typeof useTranslate>['t'];

function AvatarSourceTabs({ controller, t }: AvatarDialogContentProps) {
  return (
    <Tabs
      value={controller.source}
      onChange={(_, value: AvatarSource) => controller.handleSourceChange(value)}
      variant="fullWidth"
      sx={{ mb: 3 }}
    >
      <Tab
        value={AVATAR_SOURCE_UPLOAD}
        label={t('profile.uploadAvatar')}
        disabled={controller.saving}
      />
      <Tab
        value={AVATAR_SOURCE_ASSETS}
        label={t('profile.selectFromAssets')}
        disabled={controller.saving}
      />
    </Tabs>
  );
}

type AvatarDialogContentProps = Readonly<{
  controller: AvatarCropController;
  t: Translate;
}>;

function AvatarDialogActions({ controller, t }: AvatarDialogContentProps) {
  return (
    <DialogActions>
      <Button color="inherit" onClick={controller.handleClose} disabled={controller.saving}>
        {t('common.cancel')}
      </Button>
      <Button
        variant="contained"
        loading={controller.saving}
        disabled={!controller.canSubmit}
        onClick={controller.handleSubmit}
      >
        {t('common.save')}
      </Button>
    </DialogActions>
  );
}

function AssetSourceBody({
  controller,
  currentAvatar,
  t,
}: Readonly<{
  controller: AvatarCropController;
  currentAvatar: string;
  t: Translate;
}>) {
  if (controller.imageSrc) {
    return <AvatarCropBody controller={controller} currentAvatar={currentAvatar} t={t} />;
  }
  return (
    <Stack spacing={2}>
      <AvatarAssetPicker
        selectedId={controller.selectedAssetId}
        onSelect={controller.handleAssetSelect}
      />
      {controller.assetLoading ? (
        <Stack direction="row" spacing={1} alignItems="center">
          <CircularProgress size={18} />
          <Typography variant="body2">{t('common.loading')}</Typography>
        </Stack>
      ) : null}
      {controller.assetError ? <Alert severity="error">{controller.assetError}</Alert> : null}
    </Stack>
  );
}

type AvatarCropBodyProps = Readonly<{
  controller: AvatarCropController;
  currentAvatar: string;
  t: Translate;
}>;

function AvatarCropBody({ controller, currentAvatar, t }: AvatarCropBodyProps) {
  return (
    <Stack spacing={3}>
      <Box sx={{ height: 360, position: 'relative', bgcolor: 'grey.900', borderRadius: 2 }}>
        {controller.imageSrc ? (
          <Cropper
            image={controller.imageSrc}
            crop={controller.crop}
            zoom={controller.zoom}
            rotation={controller.rotation}
            aspect={1}
            cropShape="round"
            showGrid={false}
            onCropChange={controller.setCrop}
            onZoomChange={controller.setZoom}
            onCropComplete={controller.handleCropComplete}
          />
        ) : (
          <Stack
            alignItems="center"
            justifyContent="center"
            sx={{ height: 1, color: 'common.white' }}
          >
            <Avatar src={currentAvatar} sx={{ width: 120, height: 120, mb: 2 }} />
            <Typography>{t('profile.selectAvatarFirst')}</Typography>
          </Stack>
        )}
      </Box>
      <AvatarCropControls controller={controller} t={t} />
    </Stack>
  );
}

function AvatarCropControls({
  controller,
  t,
}: Readonly<{
  controller: AvatarCropController;
  t: Translate;
}>) {
  return (
    <Stack direction={{ xs: 'column', md: 'row' }} spacing={2} alignItems="center">
      <AvatarSourceButton controller={controller} t={t} />
      <AvatarRotationButtons controller={controller} t={t} />
      <AvatarZoomControl controller={controller} t={t} />
    </Stack>
  );
}

function AvatarSourceButton({ controller, t }: { controller: AvatarCropController; t: Translate }) {
  if (controller.source === AVATAR_SOURCE_UPLOAD) {
    return (
      <Button
        component="label"
        variant="outlined"
        disabled={controller.saving}
        startIcon={<Iconify icon="eva:cloud-upload-fill" />}
      >
        {t('actions.selectFile')}
        <input hidden type="file" accept="image/*" onChange={controller.handleFileChange} />
      </Button>
    );
  }
  return (
    <Button
      variant="outlined"
      disabled={controller.saving}
      startIcon={<Iconify icon="solar:gallery-wide-bold" />}
      onClick={controller.selectAnotherAsset}
    >
      {t('profile.selectAnotherAsset')}
    </Button>
  );
}

function AvatarRotationButtons({
  controller,
  t,
}: {
  controller: AvatarCropController;
  t: Translate;
}) {
  return (
    <Stack direction="row" spacing={1}>
      <Tooltip title={t('profile.rotateLeft')}>
        <IconButton
          aria-label={t('profile.rotateLeft')}
          onClick={controller.rotateLeft}
          disabled={controller.saving}
        >
          <Iconify icon="solar:restart-bold" />
        </IconButton>
      </Tooltip>
      <Tooltip title={t('profile.rotateRight')}>
        <IconButton
          aria-label={t('profile.rotateRight')}
          onClick={controller.rotateRight}
          disabled={controller.saving}
        >
          <Iconify icon="solar:restart-bold" sx={{ transform: 'scaleX(-1)' }} />
        </IconButton>
      </Tooltip>
    </Stack>
  );
}

function AvatarZoomControl({ controller, t }: { controller: AvatarCropController; t: Translate }) {
  return (
    <Box sx={{ flex: 1, width: 1 }}>
      <Typography variant="caption">{t('profile.zoom')}</Typography>
      <Slider
        min={MIN_ZOOM}
        max={MAX_ZOOM}
        step={ZOOM_STEP}
        value={controller.zoom}
        disabled={controller.saving}
        onChange={(_, value) => controller.setZoom(value as number)}
      />
    </Box>
  );
}
