import type {
  DayMode,
  CronParts,
  DayFieldState,
  WeekFieldState,
  YearFieldState,
  SimpleCronField,
  SimpleFieldState,
  CronBuilderState,
} from './cron-builder-model';

import {
  MIN_DAY,
  MAX_DAY,
  MAX_YEAR,
  MIN_WEEKDAY,
  MAX_WEEKDAY,
  CRON_FIELDS,
  MAX_NTH_WEEK,
  defaultWeekField,
  defaultYearField,
  defaultSimpleField,
  buildCronExpression,
  SIMPLE_FIELD_CONFIG,
} from './cron-builder-model';

export type CronEditorModel =
  | { mode: 'builder'; original: string; state: CronBuilderState; dirty: boolean }
  | { mode: 'custom'; original: string; value: string; dirty: boolean };

type Range = { min: number; max: number };

export function createCronEditorModel(expression: string, currentYear: number): CronEditorModel {
  const original = expression;
  const normalized = normalizeCronExpression(expression);
  const state = parseCronBuilderState(normalized, currentYear);
  if (state && isLosslessBuilderState(normalized, state, currentYear)) {
    return { mode: 'builder', original, state, dirty: false };
  }
  return { mode: 'custom', original, value: original, dirty: false };
}

export function cronEditorExpression(model: CronEditorModel): string {
  if (!model.dirty) return model.original;
  return model.mode === 'builder' ? buildCronExpression(model.state) : model.value;
}

export function changeCronEditorMode(
  editor: CronEditorModel,
  mode: CronEditorModel['mode'],
  currentYear: number
): CronEditorModel {
  if (editor.mode === mode) return editor;
  const value = cronEditorExpression(editor);
  if (mode === 'custom') {
    return { mode, original: editor.original, value, dirty: editor.dirty };
  }
  const parsed = createCronEditorModel(value, currentYear);
  if (parsed.mode !== 'builder') return editor;
  return { ...parsed, original: editor.original, dirty: editor.dirty };
}

export function normalizeCronExpression(expression: string): string {
  return expression.trim().split(/\s+/).filter(Boolean).join(' ');
}

export function cronExpressionParts(expression: string): CronParts {
  const values = normalizeCronExpression(expression).split(' ');
  return Object.fromEntries(
    CRON_FIELDS.map((field, index) => [field, values[index] ?? ''])
  ) as CronParts;
}

function parseCronBuilderState(expression: string, currentYear: number) {
  const values = expression.split(' ');
  if (values.length < 6 || values.length > 7) return null;
  return {
    second: parseSimpleField(values[0], 'second'),
    minute: parseSimpleField(values[1], 'minute'),
    hour: parseSimpleField(values[2], 'hour'),
    day: parseDayField(values[3]),
    month: parseSimpleField(values[4], 'month'),
    week: parseWeekField(values[5]),
    year: parseYearField(values[6] ?? '', currentYear),
  };
}

function parseSimpleField(value: string, field: SimpleCronField): SimpleFieldState {
  const defaults = defaultSimpleField(field);
  if (value === '*') return defaults;
  if (isSimpleRange(value)) return { ...defaults, mode: 'range', ...parseRange(value) };
  if (isSimpleInterval(value)) return { ...defaults, mode: 'interval', ...parseInterval(value) };
  return { ...defaults, mode: 'specific', selected: parseSelected(value) };
}

function parseDayField(value: string): DayFieldState {
  const defaults = { ...defaultSimpleField('month'), mode: 'any' as DayMode, workday: MIN_DAY };
  if (value === '*') return defaults;
  if (value === '?') return { ...defaults, mode: 'unspecified' };
  if (value === 'L') return { ...defaults, mode: 'lastDay' };
  if (/^\d+W$/.test(value)) {
    return { ...defaults, mode: 'workday', workday: Number(value.slice(0, -1)) };
  }
  if (isSimpleRange(value)) return { ...defaults, mode: 'range', ...parseRange(value) };
  if (isSimpleInterval(value)) return { ...defaults, mode: 'interval', ...parseInterval(value) };
  return { ...defaults, mode: 'specific', selected: parseSelected(value) };
}

function parseWeekField(value: string): WeekFieldState {
  const defaults = defaultWeekField();
  if (value === '*') return { ...defaults, mode: 'any' };
  if (value === '?') return defaults;
  if (/^\d+L$/.test(value)) {
    return { ...defaults, mode: 'lastWeekday', lastWeekday: Number(value.slice(0, -1)) };
  }
  if (/^\d+#\d+$/.test(value)) {
    const [nthWeekday, nth] = value.split('#').map(Number);
    return { ...defaults, mode: 'nthWeekday', nth, nthWeekday };
  }
  if (isSimpleRange(value)) return { ...defaults, mode: 'range', ...parseRange(value) };
  return { ...defaults, mode: 'specific', selected: parseSelected(value) };
}

function parseYearField(value: string, currentYear: number): YearFieldState {
  const defaults = defaultYearField(currentYear);
  if (value === '') return defaults;
  if (value === '*') return { ...defaults, mode: 'yearly' };
  if (isSimpleRange(value)) return { ...defaults, mode: 'range', ...parseRange(value) };
  if (isSimpleInterval(value)) return { ...defaults, mode: 'interval', ...parseInterval(value) };
  return { ...defaults, mode: 'specific', selected: parseSelected(value) };
}

function isLosslessBuilderState(
  expression: string,
  state: CronBuilderState,
  currentYear: number
): boolean {
  if (buildCronExpression(state) !== expression) return false;
  return (
    validSimple(state.second, SIMPLE_FIELD_CONFIG.second) &&
    validSimple(state.minute, SIMPLE_FIELD_CONFIG.minute) &&
    validSimple(state.hour, SIMPLE_FIELD_CONFIG.hour) &&
    validDay(state.day) &&
    validSimple(state.month, SIMPLE_FIELD_CONFIG.month) &&
    validWeek(state.week) &&
    validYear(state.year, { min: currentYear, max: MAX_YEAR })
  );
}

function validSimple(field: SimpleFieldState, range: Range): boolean {
  if (field.mode === 'any') return true;
  if (field.mode === 'specific') return validSelected(field.selected, range);
  if (field.mode === 'range') {
    return inRange(field.rangeStart, range) && inRange(field.rangeEnd, range);
  }
  return inRange(field.intervalStart, range) && field.intervalStep >= 1;
}

function validDay(field: DayFieldState): boolean {
  if (field.mode === 'unspecified' || field.mode === 'lastDay') return true;
  if (field.mode === 'workday') return inRange(field.workday, { min: MIN_DAY, max: MAX_DAY });
  return validSimple(field as SimpleFieldState, { min: MIN_DAY, max: MAX_DAY });
}

function validWeek(field: WeekFieldState): boolean {
  const range = { min: MIN_WEEKDAY, max: MAX_WEEKDAY };
  if (field.mode === 'unspecified') return true;
  if (field.mode === 'nthWeekday') {
    return inRange(field.nthWeekday, range) && inRange(field.nth, { min: 1, max: MAX_NTH_WEEK });
  }
  if (field.mode === 'lastWeekday') return inRange(field.lastWeekday, range);
  return validSimple(field as SimpleFieldState, range);
}

function validYear(field: YearFieldState, range: Range): boolean {
  if (field.mode === 'blank' || field.mode === 'yearly') return true;
  return validSimple(field as SimpleFieldState, range);
}

function validSelected(values: string[], range: Range): boolean {
  return (
    values.length > 0 &&
    values.every((value) => /^\d+$/.test(value) && inRange(Number(value), range))
  );
}

function inRange(value: number, range: Range): boolean {
  return Number.isInteger(value) && value >= range.min && value <= range.max;
}

function isSimpleRange(value: string): boolean {
  return /^\d+-\d+$/.test(value);
}

function isSimpleInterval(value: string): boolean {
  return /^\d+\/\d+$/.test(value);
}

function parseRange(value: string) {
  const [rangeStart, rangeEnd] = value.split('-').map(Number);
  return { rangeStart, rangeEnd };
}

function parseInterval(value: string) {
  const [intervalStart, intervalStep] = value.split('/').map(Number);
  return { intervalStart, intervalStep };
}

function parseSelected(value: string): string[] {
  return value.split(',').filter(Boolean);
}
