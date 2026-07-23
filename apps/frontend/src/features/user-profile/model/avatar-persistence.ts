import type { Area } from 'react-easy-crop';

import { uploadAccountAvatar } from '../api';
import { croppedImageBlob } from '../lib/crop-image';

const AVATAR_FILE_NAME = 'avatar.png';

type AvatarPersistenceInput = Readonly<{
  imageSrc: string;
  croppedArea: Area;
  rotation: number;
}>;

type AvatarPersistencePorts = Readonly<{
  crop: typeof croppedImageBlob;
  upload: typeof uploadAccountAvatar;
}>;

const DEFAULT_PORTS: AvatarPersistencePorts = {
  crop: croppedImageBlob,
  upload: uploadAccountAvatar,
};

export async function persistCroppedAvatar(
  input: AvatarPersistenceInput,
  ports: AvatarPersistencePorts = DEFAULT_PORTS
) {
  const blob = await ports.crop(input.imageSrc, input.croppedArea, input.rotation);
  await ports.upload(blob, AVATAR_FILE_NAME);
}
