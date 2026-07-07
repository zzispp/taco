import { fData, fNumber, fPercent } from 'src/shared/lib/format-number';

const SECONDS_PER_MINUTE = 60;
const SECONDS_PER_HOUR = 3_600;
const SECONDS_PER_DAY = 86_400;
const BITS_PER_BYTE = 8;
const MEGABIT = 1_000_000;

export type DurationLabels = {
  day: string;
  hour: string;
  minute: string;
};

export function formatBytes(value: number) {
  return fData(value);
}

export function formatPercent(value: number) {
  return fPercent(value);
}

export function formatNumber(value: number) {
  return fNumber(value);
}

export function formatBandwidth(bytesPerSecond: number) {
  return `${fNumber((bytesPerSecond * BITS_PER_BYTE) / MEGABIT)} Mbps`;
}

export function formatDuration(totalSeconds: number, labels: DurationLabels) {
  const days = Math.floor(totalSeconds / SECONDS_PER_DAY);
  const hours = Math.floor((totalSeconds % SECONDS_PER_DAY) / SECONDS_PER_HOUR);
  const minutes = Math.floor((totalSeconds % SECONDS_PER_HOUR) / SECONDS_PER_MINUTE);

  if (days > 0) {
    return `${days}${labels.day} ${hours}${labels.hour} ${minutes}${labels.minute}`;
  }

  if (hours > 0) {
    return `${hours}${labels.hour} ${minutes}${labels.minute}`;
  }

  return `${minutes}${labels.minute}`;
}

export function formatUnixSeconds(value: number, locale?: string) {
  return new Date(value * 1000).toLocaleString(locale);
}

export function formatDateTime(value: string, locale?: string) {
  return new Date(value).toLocaleString(locale);
}
