'use client';

import type { Area } from 'react-easy-crop';

import Cropper from 'react-easy-crop';
import { useState, useCallback } from 'react';

import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Slider from '@mui/material/Slider';
import Dialog from '@mui/material/Dialog';
import Avatar from '@mui/material/Avatar';
import Typography from '@mui/material/Typography';
import DialogTitle from '@mui/material/DialogTitle';
import DialogContent from '@mui/material/DialogContent';
import DialogActions from '@mui/material/DialogActions';

import { toast } from 'src/shared/ui/snackbar';
import { Iconify } from 'src/shared/ui/iconify';
import { useTranslate } from 'src/shared/i18n/use-locales';

import { croppedImageBlob, uploadAccountAvatar } from 'src/features/user-profile';

const MIN_ZOOM = 1;
const MAX_ZOOM = 3;
const ROTATE_STEP = 90;

export type AvatarCropDialogProps = {
  open: boolean;
  currentAvatar: string;
  onClose: () => void;
  onUploaded: () => Promise<void>;
};

export function AvatarCropDialog({
  open,
  currentAvatar,
  onClose,
  onUploaded,
}: AvatarCropDialogProps) {
  const { t } = useTranslate('admin');
  const state = useAvatarCropState();
  const actions = useAvatarCropActions({ state, onClose, onUploaded, t });

  return (
    <Dialog fullWidth maxWidth="md" open={open} onClose={onClose}>
      <DialogTitle>{t('profile.changeAvatar')}</DialogTitle>
      <DialogContent>
        <AvatarCropBody {...{ state, actions, currentAvatar, t }} />
      </DialogContent>
      <DialogActions>
        <Button color="inherit" onClick={onClose}>
          {t('common.cancel')}
        </Button>
        <Button
          variant="contained"
          loading={state.loading}
          disabled={!state.imageSrc}
          onClick={actions.handleSubmit}
        >
          {t('common.save')}
        </Button>
      </DialogActions>
    </Dialog>
  );
}

function useAvatarCropState() {
  const [imageSrc, setImageSrc] = useState('');
  const [fileName, setFileName] = useState('avatar.png');
  const [crop, setCrop] = useState({ x: 0, y: 0 });
  const [zoom, setZoom] = useState(MIN_ZOOM);
  const [rotation, setRotation] = useState(0);
  const [croppedArea, setCroppedArea] = useState<Area | null>(null);
  const [loading, setLoading] = useState(false);

  return {
    imageSrc,
    fileName,
    crop,
    zoom,
    rotation,
    croppedArea,
    loading,
    setImageSrc,
    setFileName,
    setCrop,
    setZoom,
    setRotation,
    setCroppedArea,
    setLoading,
  };
}

type AvatarCropActionsOptions = Readonly<{
  state: ReturnType<typeof useAvatarCropState>;
  onClose: () => void;
  onUploaded: () => Promise<void>;
  t: ReturnType<typeof useTranslate>['t'];
}>;

function useAvatarCropActions({ state, onClose, onUploaded, t }: AvatarCropActionsOptions) {
  const handleCropComplete = useCallback(
    (_area: Area, pixels: Area) => state.setCroppedArea(pixels),
    [state]
  );
  const handleFileChange = useCallback(
    (event: React.ChangeEvent<HTMLInputElement>) => {
      const file = event.target.files?.[0];
      event.target.value = '';
      if (!file) return;
      if (!file.type.startsWith('image/')) {
        toast.error(t('profile.avatarImageOnly'));
        return;
      }
      state.setFileName(file.name || 'avatar.png');
      state.setImageSrc(URL.createObjectURL(file));
      state.setCrop({ x: 0, y: 0 });
      state.setZoom(MIN_ZOOM);
      state.setRotation(0);
    },
    [state, t]
  );
  const handleSubmit = useCallback(async () => {
    if (!state.imageSrc || !state.croppedArea) return;
    state.setLoading(true);
    try {
      const blob = await croppedImageBlob(state.imageSrc, state.croppedArea, state.rotation);
      await uploadAccountAvatar(blob, state.fileName);
      await onUploaded();
      toast.success(t('messages.saved'));
      onClose();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      state.setLoading(false);
    }
  }, [onClose, onUploaded, state, t]);

  return { handleCropComplete, handleFileChange, handleSubmit };
}

type AvatarCropBodyProps = Readonly<{
  state: ReturnType<typeof useAvatarCropState>;
  actions: ReturnType<typeof useAvatarCropActions>;
  currentAvatar: string;
  t: ReturnType<typeof useTranslate>['t'];
}>;

function AvatarCropBody({ state, actions, currentAvatar, t }: AvatarCropBodyProps) {
  return (
    <Stack spacing={3}>
      <AvatarCropCanvas {...{ state, actions, currentAvatar, t }} />
      <AvatarCropControls {...{ state, actions, t }} />
    </Stack>
  );
}

function AvatarCropCanvas({ state, actions, currentAvatar, t }: AvatarCropBodyProps) {
  return (
    <Box sx={{ height: 360, position: 'relative', bgcolor: 'grey.900', borderRadius: 2 }}>
      {state.imageSrc ? (
        <Cropper
          image={state.imageSrc}
          crop={state.crop}
          zoom={state.zoom}
          rotation={state.rotation}
          aspect={1}
          cropShape="round"
          showGrid={false}
          onCropChange={state.setCrop}
          onZoomChange={state.setZoom}
          onCropComplete={actions.handleCropComplete}
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
  );
}

function AvatarCropControls({ state, actions, t }: Omit<AvatarCropBodyProps, 'currentAvatar'>) {
  return (
    <Stack direction={{ xs: 'column', md: 'row' }} spacing={2} alignItems="center">
      <Button
        component="label"
        variant="outlined"
        startIcon={<Iconify icon="eva:cloud-upload-fill" />}
      >
        {t('actions.selectFile')}
        <input hidden type="file" accept="image/*" onChange={actions.handleFileChange} />
      </Button>
      <Stack direction="row" spacing={1}>
        <Button
          variant="outlined"
          onClick={() => state.setRotation((value) => value - ROTATE_STEP)}
        >
          ↺
        </Button>
        <Button
          variant="outlined"
          onClick={() => state.setRotation((value) => value + ROTATE_STEP)}
        >
          ↻
        </Button>
      </Stack>
      <Box sx={{ flex: 1, width: 1 }}>
        <Typography variant="caption">{t('profile.zoom')}</Typography>
        <Slider
          min={MIN_ZOOM}
          max={MAX_ZOOM}
          step={0.1}
          value={state.zoom}
          onChange={(_, value) => state.setZoom(value as number)}
        />
      </Box>
    </Stack>
  );
}
