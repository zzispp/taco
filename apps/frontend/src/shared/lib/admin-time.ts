import type { DateInput } from './format-time';

import dayjs from 'dayjs';

const ADMIN_DATE_TIME_FORMAT = 'YYYY-MM-DD HH:mm:ss';
const INVALID_DATE = 'Invalid';

export function fAdminDateTime(input: DateInput): string {
  if (!input) return '';
  const date = dayjs(input);
  if (!date.isValid()) return INVALID_DATE;
  return date.format(ADMIN_DATE_TIME_FORMAT);
}
