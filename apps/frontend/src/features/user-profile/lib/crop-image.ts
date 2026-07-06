import type { Area } from 'react-easy-crop';

const HALF_TURN = 180;

export async function croppedImageBlob(imageSrc: string, crop: Area, rotation: number) {
  const image = await createImage(imageSrc);
  const canvas = document.createElement('canvas');
  const context = requiredContext(canvas);
  const rotated = rotatedSize(image.width, image.height, rotation);
  canvas.width = rotated.width;
  canvas.height = rotated.height;
  drawRotatedImage(context, image, rotated, rotation);

  const cropped = context.getImageData(crop.x, crop.y, crop.width, crop.height);
  canvas.width = crop.width;
  canvas.height = crop.height;
  requiredContext(canvas).putImageData(cropped, 0, 0);
  return canvasBlob(canvas);
}

function createImage(src: string): Promise<HTMLImageElement> {
  return new Promise((resolve, reject) => {
    const image = new Image();
    image.addEventListener('load', () => resolve(image));
    image.addEventListener('error', () => reject(new Error('Failed to load image')));
    image.src = src;
  });
}

function requiredContext(canvas: HTMLCanvasElement) {
  const context = canvas.getContext('2d');
  if (!context) {
    throw new Error('Canvas context is not available');
  }
  return context;
}

function drawRotatedImage(
  context: CanvasRenderingContext2D,
  image: HTMLImageElement,
  rotated: Size,
  rotation: number
) {
  context.translate(rotated.width / 2, rotated.height / 2);
  context.rotate((rotation * Math.PI) / HALF_TURN);
  context.drawImage(image, -image.width / 2, -image.height / 2);
}

function rotatedSize(width: number, height: number, rotation: number): Size {
  const radians = (rotation * Math.PI) / HALF_TURN;
  return {
    width: Math.abs(Math.cos(radians) * width) + Math.abs(Math.sin(radians) * height),
    height: Math.abs(Math.sin(radians) * width) + Math.abs(Math.cos(radians) * height),
  };
}

function canvasBlob(canvas: HTMLCanvasElement): Promise<Blob> {
  return new Promise((resolve, reject) => {
    canvas.toBlob(
      (blob) => (blob ? resolve(blob) : reject(new Error('Failed to crop image'))),
      'image/png'
    );
  });
}

type Size = {
  width: number;
  height: number;
};
