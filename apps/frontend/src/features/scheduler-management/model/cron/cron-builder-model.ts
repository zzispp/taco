export const CRON_FIELDS = ['second', 'minute', 'hour', 'day', 'month', 'week', 'year'] as const;
export const SIMPLE_CRON_FIELDS = ['second', 'minute', 'hour', 'month'] as const;

export type CronField = (typeof CRON_FIELDS)[number];
export type SimpleCronField = (typeof SIMPLE_CRON_FIELDS)[number];
export type SimpleMode = 'any' | 'range' | 'interval' | 'specific';
export type DayMode = SimpleMode | 'unspecified' | 'workday' | 'lastDay';
export type WeekMode = SimpleMode | 'unspecified' | 'nthWeekday' | 'lastWeekday';
export type YearMode = 'blank' | 'yearly' | 'range' | 'interval' | 'specific';

export type CronParts = Record<CronField, string>;

export type SimpleFieldState = {
  mode: SimpleMode;
  rangeStart: number;
  rangeEnd: number;
  intervalStart: number;
  intervalStep: number;
  selected: string[];
};

export type DayFieldState = Omit<SimpleFieldState, 'mode'> & {
  mode: DayMode;
  workday: number;
};

export type WeekFieldState = Omit<SimpleFieldState, 'mode'> & {
  mode: WeekMode;
  nth: number;
  nthWeekday: number;
  lastWeekday: number;
};

export type YearFieldState = {
  mode: YearMode;
  rangeStart: number;
  rangeEnd: number;
  intervalStart: number;
  intervalStep: number;
  selected: string[];
};

export type CronBuilderState = {
  second: SimpleFieldState;
  minute: SimpleFieldState;
  hour: SimpleFieldState;
  day: DayFieldState;
  month: SimpleFieldState;
  week: WeekFieldState;
  year: YearFieldState;
};

type SimpleFieldConfig = {
  min: number;
  max: number;
  rangeStartMax: number;
  defaultRangeStart: number;
  defaultRangeEnd: number;
  defaultIntervalStart: number;
  defaultIntervalStep: number;
};

const MIN_SECONDS = 0;
const MAX_SECONDS = 59;
const MAX_SECOND_RANGE_START = 58;
const MIN_HOUR = 0;
const MAX_HOUR = 23;
const MAX_HOUR_RANGE_START = 22;
export const MIN_DAY = 1;
export const MAX_DAY = 31;
const MAX_DAY_RANGE_START = 30;
const MIN_MONTH = 1;
const MAX_MONTH = 12;
const MAX_MONTH_RANGE_START = 11;
export const MIN_WEEKDAY = 1;
export const MAX_WEEKDAY = 7;
export const MAX_NTH_WEEK = 4;
export const MAX_YEAR = 2099;
export const MAX_YEAR_RANGE_START = 2098;

export const SIMPLE_FIELD_CONFIG: Record<SimpleCronField, SimpleFieldConfig> = {
  second: makeConfig(MIN_SECONDS, MAX_SECONDS, MAX_SECOND_RANGE_START),
  minute: makeConfig(MIN_SECONDS, MAX_SECONDS, MAX_SECOND_RANGE_START),
  hour: makeConfig(MIN_HOUR, MAX_HOUR, MAX_HOUR_RANGE_START),
  month: makeConfig(MIN_MONTH, MAX_MONTH, MAX_MONTH_RANGE_START),
};

export const DAY_CONFIG = makeConfig(MIN_DAY, MAX_DAY, MAX_DAY_RANGE_START);
export const WEEKDAY_VALUES = [2, 3, 4, 5, 6, 7, 1] as const;

export function defaultCronState(currentYear: number): CronBuilderState {
  return {
    second: defaultSimpleField('second'),
    minute: defaultSimpleField('minute'),
    hour: defaultSimpleField('hour'),
    day: { ...defaultSimpleField('month'), mode: 'any', workday: MIN_DAY },
    month: defaultSimpleField('month'),
    week: defaultWeekField(),
    year: defaultYearField(currentYear),
  };
}

export function buildCronParts(state: CronBuilderState): CronParts {
  return {
    second: simpleValue(state.second),
    minute: simpleValue(state.minute),
    hour: simpleValue(state.hour),
    day: dayValue(state.day),
    month: simpleValue(state.month),
    week: weekValue(state.week),
    year: yearValue(state.year),
  };
}

export function buildCronExpression(state: CronBuilderState): string {
  const parts = buildCronParts(state);
  const base = [parts.second, parts.minute, parts.hour, parts.day, parts.month, parts.week];
  return parts.year ? [...base, parts.year].join(' ') : base.join(' ');
}

export function updateSimpleField(
  state: CronBuilderState,
  field: SimpleCronField,
  patch: Partial<SimpleFieldState>
): CronBuilderState {
  const next = { ...state, [field]: { ...state[field], ...patch } } as CronBuilderState;
  return field === 'hour' ? applyHourDependencies(next) : next;
}

export function updateDayField(
  state: CronBuilderState,
  patch: Partial<DayFieldState>
): CronBuilderState {
  const next: CronBuilderState = { ...state, day: { ...state.day, ...patch } as DayFieldState };
  if (dayValue(next.day) === '?' || weekValue(next.week) === '?') return next;
  return { ...next, week: { ...next.week, mode: 'unspecified' } };
}

export function updateWeekField(
  state: CronBuilderState,
  patch: Partial<WeekFieldState>
): CronBuilderState {
  const next: CronBuilderState = { ...state, week: { ...state.week, ...patch } as WeekFieldState };
  if (weekValue(next.week) === '?' || dayValue(next.day) === '?') return next;
  return { ...next, day: { ...next.day, mode: 'unspecified' } };
}

export function updateYearField(
  state: CronBuilderState,
  patch: Partial<YearFieldState>
): CronBuilderState {
  return { ...state, year: { ...state.year, ...patch } };
}

export function numberOptions(min: number, max: number): string[] {
  return Array.from({ length: max - min + 1 }, (_, index) => String(min + index));
}

export function yearOptions(currentYear: number): string[] {
  return numberOptions(currentYear, currentYear + 8);
}

export function clampNumber(value: number, min: number, max: number): number {
  if (!Number.isFinite(value)) return min;
  return Math.min(Math.max(Math.trunc(value), min), max);
}

function makeConfig(min: number, max: number, rangeStartMax: number): SimpleFieldConfig {
  return {
    min,
    max,
    rangeStartMax,
    defaultRangeStart: min,
    defaultRangeEnd: min + 1,
    defaultIntervalStart: min,
    defaultIntervalStep: 1,
  };
}

export function defaultSimpleField(field: SimpleCronField): SimpleFieldState {
  const config = SIMPLE_FIELD_CONFIG[field];
  return {
    mode: 'any',
    rangeStart: config.defaultRangeStart,
    rangeEnd: config.defaultRangeEnd,
    intervalStart: config.defaultIntervalStart,
    intervalStep: config.defaultIntervalStep,
    selected: [],
  };
}

export function defaultWeekField(): WeekFieldState {
  return {
    ...defaultSimpleField('month'),
    mode: 'unspecified',
    rangeStart: 2,
    rangeEnd: 3,
    nth: 1,
    nthWeekday: 2,
    lastWeekday: 2,
  };
}

export function defaultYearField(currentYear: number): YearFieldState {
  return {
    mode: 'blank',
    rangeStart: currentYear,
    rangeEnd: currentYear + 1,
    intervalStart: currentYear,
    intervalStep: 1,
    selected: [],
  };
}

function simpleValue(field: SimpleFieldState): string {
  if (field.mode === 'any') return '*';
  if (field.mode === 'range') return `${field.rangeStart}-${field.rangeEnd}`;
  if (field.mode === 'interval') return `${field.intervalStart}/${field.intervalStep}`;
  return joinSelected(field.selected, '*');
}

function dayValue(field: DayFieldState): string {
  if (field.mode === 'unspecified') return '?';
  if (field.mode === 'workday') return `${field.workday}W`;
  if (field.mode === 'lastDay') return 'L';
  return simpleValue(field as SimpleFieldState);
}

function weekValue(field: WeekFieldState): string {
  if (field.mode === 'unspecified') return '?';
  if (field.mode === 'nthWeekday') return `${field.nthWeekday}#${field.nth}`;
  if (field.mode === 'lastWeekday') return `${field.lastWeekday}L`;
  return simpleValue(field as SimpleFieldState);
}

function yearValue(field: YearFieldState): string {
  if (field.mode === 'blank') return '';
  if (field.mode === 'yearly') return '*';
  if (field.mode === 'range') return `${field.rangeStart}-${field.rangeEnd}`;
  if (field.mode === 'interval') return `${field.intervalStart}/${field.intervalStep}`;
  return joinSelected(field.selected, '');
}

function joinSelected(selected: string[], emptyValue: string): string {
  return selected.length ? selected.join(',') : emptyValue;
}

function applyHourDependencies(state: CronBuilderState): CronBuilderState {
  const minute =
    simpleValue(state.minute) === '*'
      ? { ...state.minute, mode: 'specific' as const, selected: ['0'] }
      : state.minute;
  const second =
    simpleValue(state.second) === '*'
      ? { ...state.second, mode: 'specific' as const, selected: ['0'] }
      : state.second;
  return { ...state, minute, second };
}
