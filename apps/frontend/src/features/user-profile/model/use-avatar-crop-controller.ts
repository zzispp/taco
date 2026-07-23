'use client';

import type { Area, Point } from 'react-easy-crop';
import type { FileEntry } from 'src/entities/file';

import { useRef, useEffect, useReducer, useCallback } from 'react';

import { toast } from 'src/shared/ui/snackbar';
import { apiMutationErrorMessage } from 'src/shared/api/mutation-error';

import { persistCroppedAvatar } from './avatar-persistence';
import { loadAvatarAssetPreview } from './avatar-asset-source';
import { type ObjectUrlLifecycle, createObjectUrlLifecycle } from '../lib/object-url-lifecycle';

const MIN_ZOOM = 1;
const ROTATE_STEP = 90;
export const AVATAR_SOURCE_UPLOAD = 'upload';
export const AVATAR_SOURCE_ASSETS = 'assets';

export type AvatarSource = typeof AVATAR_SOURCE_UPLOAD | typeof AVATAR_SOURCE_ASSETS;

type Translate = (key: string, options?: Record<string, unknown>) => string;

type AvatarCropState = Readonly<{
  source: AvatarSource;
  imageSrc: string;
  selectedAssetId: string | null;
  crop: Point;
  zoom: number;
  rotation: number;
  croppedArea: Area | null;
  saving: boolean;
  assetLoading: boolean;
  assetError: string | null;
}>;

type CropStateAction =
  | Readonly<{ type: 'patch'; patch: Readonly<Partial<AvatarCropState>> }>
  | Readonly<{ type: 'reset'; source: AvatarSource }>;

type ControllerOptions = Readonly<{
  open: boolean;
  onClose: () => void;
  onUploaded: () => Promise<void>;
  t: Translate;
}>;

export function useAvatarCropController(options: ControllerOptions) {
  const cropState = useAvatarCropState();
  const assetSelection = useAvatarAssetSelection(cropState, options.t);
  const actions = useAvatarCropActions({ options, cropState, assetSelection });
  const { reset } = cropState;
  const { abort } = assetSelection;
  useEffect(() => {
    if (options.open) return;
    abort();
    reset(AVATAR_SOURCE_UPLOAD);
  }, [abort, options.open, reset]);
  return {
    ...cropState.value,
    canSubmit: canSubmitAvatar(cropState.value),
    ...actions,
    handleAssetSelect: assetSelection.select,
    selectAnotherAsset: assetSelection.clear,
  };
}

function useAvatarCropState() {
  const [value, dispatch] = useReducer(
    avatarCropReducer,
    AVATAR_SOURCE_UPLOAD,
    createAvatarCropState
  );
  const objectUrls = useRef<ObjectUrlLifecycle | null>(null);
  if (!objectUrls.current) objectUrls.current = createObjectUrlLifecycle(URL);
  const patch = useCallback((next: Readonly<Partial<AvatarCropState>>) => {
    dispatch({ type: 'patch', patch: next });
  }, []);
  const clearImage = useCallback(() => {
    objectUrls.current?.clear();
    patch({ imageSrc: '', croppedArea: null });
  }, [patch]);
  const installImage = useCallback(
    (blob: Blob, next: Readonly<Partial<AvatarCropState>> = {}) => {
      const imageSrc = objectUrls.current?.replace(blob) ?? '';
      patch({ ...initialCrop(), ...next, imageSrc });
    },
    [patch]
  );
  const reset = useCallback((source: AvatarSource) => {
    objectUrls.current?.clear();
    dispatch({ type: 'reset', source });
  }, []);
  useEffect(() => () => objectUrls.current?.clear(), []);
  return { value, patch, clearImage, installImage, reset };
}

type CropState = ReturnType<typeof useAvatarCropState>;

function useAvatarAssetSelection(state: CropState, t: Translate) {
  const requestRef = useRef<AbortController | null>(null);
  const abort = useCallback(() => {
    requestRef.current?.abort();
    requestRef.current = null;
  }, []);
  const clear = useCallback(() => {
    abort();
    state.reset(AVATAR_SOURCE_ASSETS);
  }, [abort, state]);
  const select = useCallback(
    async (entry: FileEntry) => {
      abort();
      state.clearImage();
      const request = new AbortController();
      requestRef.current = request;
      state.patch({ selectedAssetId: entry.id, assetLoading: true, assetError: null });
      try {
        const blob = await loadAvatarAssetPreview(entry.id, request.signal);
        if (request.signal.aborted) return;
        if (!blob.type.startsWith('image/')) {
          state.patch({ assetLoading: false, assetError: t('profile.assetContentInvalid') });
          return;
        }
        state.installImage(blob, { selectedAssetId: entry.id, assetLoading: false });
      } catch (error) {
        if (!request.signal.aborted) {
          state.patch({
            assetLoading: false,
            assetError: apiMutationErrorMessage(error, t('profile.assetContentLoadFailed')),
          });
        }
      } finally {
        if (requestRef.current === request) requestRef.current = null;
      }
    },
    [abort, state, t]
  );
  useEffect(() => abort, [abort]);
  return { abort, clear, select };
}

type AssetSelection = ReturnType<typeof useAvatarAssetSelection>;

function useAvatarCropActions(
  config: Readonly<{
    options: ControllerOptions;
    cropState: CropState;
    assetSelection: AssetSelection;
  }>
) {
  const { options, cropState, assetSelection } = config;
  const handleSourceChange = useCallback(
    (source: AvatarSource) => {
      if (cropState.value.saving || source === cropState.value.source) return;
      assetSelection.abort();
      cropState.reset(source);
    },
    [assetSelection, cropState]
  );
  const handleFileChange = useCallback(
    (event: React.ChangeEvent<HTMLInputElement>) => {
      const file = event.target.files?.[0];
      event.target.value = '';
      if (!file || cropState.value.saving) return;
      if (!file.type.startsWith('image/')) {
        toast.error(options.t('profile.avatarImageOnly'));
        return;
      }
      assetSelection.abort();
      cropState.installImage(file, {
        selectedAssetId: null,
        assetLoading: false,
        assetError: null,
      });
    },
    [assetSelection, cropState, options]
  );
  const handleSubmit = useAvatarSubmit(options, cropState);
  return {
    handleSourceChange,
    handleFileChange,
    handleSubmit,
    handleClose: () => {
      if (cropState.value.saving) return;
      assetSelection.abort();
      cropState.reset(AVATAR_SOURCE_UPLOAD);
      options.onClose();
    },
    ...createCropControlActions(cropState),
  };
}

function createCropControlActions(cropState: CropState) {
  return {
    handleCropComplete: (_area: Area, croppedArea: Area) => cropState.patch({ croppedArea }),
    setCrop: (crop: Point) => cropState.patch({ crop }),
    setZoom: (zoom: number) => cropState.patch({ zoom }),
    rotateLeft: () => cropState.patch({ rotation: cropState.value.rotation - ROTATE_STEP }),
    rotateRight: () => cropState.patch({ rotation: cropState.value.rotation + ROTATE_STEP }),
  };
}

function useAvatarSubmit(options: ControllerOptions, state: CropState) {
  return useCallback(async () => {
    const { imageSrc, croppedArea, rotation } = state.value;
    if (!imageSrc || !croppedArea || state.value.assetLoading || state.value.saving) return;
    state.patch({ saving: true });
    try {
      await persistCroppedAvatar({ imageSrc, croppedArea, rotation });
      await options.onUploaded();
      toast.success(options.t('messages.saved'));
      state.reset(AVATAR_SOURCE_UPLOAD);
      options.onClose();
    } catch (error) {
      toast.error(apiMutationErrorMessage(error, options.t('messages.saveFailed')));
    } finally {
      state.patch({ saving: false });
    }
  }, [options, state]);
}

function createAvatarCropState(source: AvatarSource): AvatarCropState {
  return {
    source,
    imageSrc: '',
    selectedAssetId: null,
    ...initialCrop(),
    saving: false,
    assetLoading: false,
    assetError: null,
  };
}

function initialCrop() {
  return { crop: { x: 0, y: 0 }, zoom: MIN_ZOOM, rotation: 0, croppedArea: null };
}

function avatarCropReducer(state: AvatarCropState, action: CropStateAction): AvatarCropState {
  if (action.type === 'reset') return createAvatarCropState(action.source);
  return { ...state, ...action.patch };
}

function canSubmitAvatar(state: AvatarCropState) {
  return Boolean(state.imageSrc && state.croppedArea && !state.assetLoading && !state.saving);
}

export type AvatarCropController = ReturnType<typeof useAvatarCropController>;
