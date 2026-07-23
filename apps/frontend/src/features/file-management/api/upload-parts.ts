import type { UploadPartReceipt } from 'src/entities/file';

const SHA256_PATTERN = /^[0-9a-f]{64}$/;

export function completeParts(
  config: Readonly<{
    receipts: ReadonlyArray<UploadPartReceipt>;
    fileSize: number;
    partSize: number;
  }>
): UploadPartReceipt[] {
  const receipts = receiptMap(config.receipts, config.fileSize, config.partSize);
  const partCount = Math.ceil(config.fileSize / config.partSize);
  return Array.from({ length: partCount }, (_, index) => {
    const partNumber = index + 1;
    const receipt = receipts.get(partNumber);
    if (!receipt) {
      throw new Error(`Upload part ${partNumber} is missing or has an unexpected size`);
    }
    return receipt;
  });
}

export function partLength(fileSize: number, partSize: number, partNumber: number): number {
  const start = (partNumber - 1) * partSize;
  return Math.min(partSize, fileSize - start);
}

export function uploadedBytes(parts: Iterable<UploadPartReceipt>): number {
  return Array.from(parts).reduce((total, part) => total + part.size_bytes, 0);
}

export function receiptMap(
  parts: ReadonlyArray<UploadPartReceipt>,
  fileSize: number,
  partSize: number
): Map<number, UploadPartReceipt> {
  const result = new Map<number, UploadPartReceipt>();
  const partCount = Math.ceil(fileSize / partSize);
  for (const part of parts) {
    const expectedSize = partLength(fileSize, partSize, part.part_number);
    const validNumber = Number.isInteger(part.part_number) && part.part_number > 0;
    if (!validNumber || part.part_number > partCount || part.size_bytes !== expectedSize) {
      throw new Error(`Upload part ${part.part_number} is invalid`);
    }
    if (!SHA256_PATTERN.test(part.sha256)) {
      throw new Error(`Upload part ${part.part_number} has an invalid digest`);
    }
    if (result.has(part.part_number)) {
      throw new Error(`Upload part ${part.part_number} is duplicated`);
    }
    result.set(part.part_number, part);
  }
  return result;
}

export function validateReceipt(
  config: Readonly<{
    receipt: UploadPartReceipt;
    partNumber: number;
    expectedSize: number;
    digest: string;
  }>
) {
  if (
    config.receipt.part_number !== config.partNumber ||
    config.receipt.size_bytes !== config.expectedSize ||
    config.receipt.sha256 !== config.digest
  ) {
    throw new Error(`Upload part ${config.partNumber} response does not match the request`);
  }
}
