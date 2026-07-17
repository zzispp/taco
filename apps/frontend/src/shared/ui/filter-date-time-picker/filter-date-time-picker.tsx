import type { Dayjs } from 'dayjs';

import dayjs from 'dayjs';

import { DateTimePicker } from '@mui/x-date-pickers/DateTimePicker';

import { INVALID_LOCAL_DATE_TIME_DRAFT } from 'src/shared/lib/local-date-time-filter';

const FILTER_DATE_TIME_WIDTH = 220;
const LOCAL_DATE_TIME_DRAFT_FORMAT = 'YYYY-MM-DDTHH:mm:ss.SSS';
const LOCAL_DATE_TIME_DISPLAY_FORMAT = 'YYYY-MM-DD HH:mm:ss';
const DATE_TIME_VIEWS = ['year', 'month', 'day', 'hours', 'minutes', 'seconds'] as const;

export function FilterDateTimePicker({ label, value, onChange }: FilterDateTimePickerProps) {
  return (
    <DateTimePicker
      label={label}
      format={LOCAL_DATE_TIME_DISPLAY_FORMAT}
      timeSteps={{ minutes: 1, seconds: 1 }}
      value={value ? dayjs(value) : null}
      views={DATE_TIME_VIEWS}
      slotProps={{
        textField: {
          size: 'small',
          sx: { width: { xs: '100%', sm: FILTER_DATE_TIME_WIDTH } },
        },
      }}
      onChange={(nextValue) => onChange(toLocalDateTimeDraft(nextValue))}
    />
  );
}

function toLocalDateTimeDraft(value: Dayjs | null) {
  if (!value) return '';
  return value.isValid()
    ? value.format(LOCAL_DATE_TIME_DRAFT_FORMAT)
    : INVALID_LOCAL_DATE_TIME_DRAFT;
}

type FilterDateTimePickerProps = Readonly<{
  label: string;
  value: string;
  onChange: (value: string) => void;
}>;
