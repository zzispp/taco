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

import { uploadAccountAvatar } from 'src/features/user-profile';
import { croppedImageBlob } from 'src/features/user-profile/lib/crop-image';

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
  const [imageSrc, setImageSrc] = useState('');
  const [fileName, setFileName] = useState('avatar.png');
  const [crop, setCrop] = useState({ x: 0, y: 0 });
  const [zoom, setZoom] = useState(MIN_ZOOM);
  const [rotation, setRotation] = useState(0);
  const [croppedArea, setCroppedArea] = useState<Area | null>(null);
  const [loading, setLoading] = useState(false);

  const handleCropComplete = useCallback((_area: Area, pixels: Area) => setCroppedArea(pixels), []);

  const handleFileChange = useCallback(
    (event: React.ChangeEvent<HTMLInputElement>) => {
      const file = event.target.files?.[0];
      event.target.value = '';
      if (!file) return;
      if (!file.type.startsWith('image/')) {
        toast.error(t('profile.avatarImageOnly'));
        return;
      }
      setFileName(file.name || 'avatar.png');
      setImageSrc(URL.createObjectURL(file));
      setCrop({ x: 0, y: 0 });
      setZoom(MIN_ZOOM);
      setRotation(0);
    },
    [t]
  );

  const handleSubmit = useCallback(async () => {
    if (!imageSrc || !croppedArea) return;
    setLoading(true);
    try {
      const blob = await croppedImageBlob(imageSrc, croppedArea, rotation);
      await uploadAccountAvatar(blob, fileName);
      await onUploaded();
      toast.success(t('messages.saved'));
      onClose();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setLoading(false);
    }
  }, [croppedArea, fileName, imageSrc, onClose, onUploaded, rotation, t]);

  return (
    <Dialog fullWidth maxWidth="md" open={open} onClose={onClose}>
      <DialogTitle>{t('profile.changeAvatar')}</DialogTitle>
      <DialogContent>
        <Stack spacing={3}>
          <Box sx={{ height: 360, position: 'relative', bgcolor: 'grey.900', borderRadius: 2 }}>
            {imageSrc ? (
              <Cropper
                image={imageSrc}
                crop={crop}
                zoom={zoom}
                rotation={rotation}
                aspect={1}
                cropShape="round"
                showGrid={false}
                onCropChange={setCrop}
                onZoomChange={setZoom}
                onCropComplete={handleCropComplete}
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

          <Stack direction={{ xs: 'column', md: 'row' }} spacing={2} alignItems="center">
            <Button
              component="label"
              variant="outlined"
              startIcon={<Iconify icon="eva:cloud-upload-fill" />}
            >
              {t('actions.selectFile')}
              <input hidden type="file" accept="image/*" onChange={handleFileChange} />
            </Button>
            <Stack direction="row" spacing={1}>
              <Button
                variant="outlined"
                onClick={() => setRotation((value) => value - ROTATE_STEP)}
              >
                ↺
              </Button>
              <Button
                variant="outlined"
                onClick={() => setRotation((value) => value + ROTATE_STEP)}
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
                value={zoom}
                onChange={(_, value) => setZoom(value as number)}
              />
            </Box>
          </Stack>
        </Stack>
      </DialogContent>
      <DialogActions>
        <Button color="inherit" onClick={onClose}>
          {t('common.cancel')}
        </Button>
        <Button variant="contained" loading={loading} disabled={!imageSrc} onClick={handleSubmit}>
          {t('common.save')}
        </Button>
      </DialogActions>
    </Dialog>
  );
}
