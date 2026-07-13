import type { ParamFieldSpec } from 'src/entities/scheduler';

import { PARAM_WIDGET } from 'src/entities/scheduler';

export type ParamValues = Record<string, unknown>;

export type KeyValueDraftRow = Readonly<{
  id: string;
  key: string;
  value: string;
}>;

export type ParamDraft = Readonly<{
  values: ParamValues;
  json: Readonly<Record<string, string>>;
  keyValues: Readonly<Record<string, readonly KeyValueDraftRow[]>>;
  invalidKeyValues: ReadonlySet<string>;
  present: ReadonlySet<string>;
}>;

export type ParamDraftErrorCode =
  'duplicate_key' | 'empty_key' | 'invalid_json' | 'invalid_key_value';

type InitialKeyValueDraft = Readonly<{
  path: string;
  rows: readonly KeyValueDraftRow[];
  invalid: boolean;
}>;

export class ParamDraftError extends Error {
  constructor(
    readonly code: ParamDraftErrorCode,
    readonly path: string
  ) {
    super(code);
    this.name = 'ParamDraftError';
  }
}

export function createParamDraft(
  values: ParamValues,
  fields: ParamFieldSpec[],
  createId: () => string
): ParamDraft {
  const json = Object.fromEntries(
    fields
      .filter((field) => field.widget === PARAM_WIDGET.JSON_EDITOR)
      .map((field) => [
        field.path,
        JSON.stringify(values[field.path] === undefined ? {} : values[field.path], null, 2),
      ])
  );
  const keyValueDrafts = fields
    .filter((field) => field.widget === PARAM_WIDGET.KEY_VALUE)
    .map((field) => createInitialKeyValueDraft(field, values, createId));
  const keyValues = Object.fromEntries(keyValueDrafts.map((draft) => [draft.path, draft.rows]));
  const invalidKeyValues = new Set(
    keyValueDrafts.filter((draft) => draft.invalid).map((draft) => draft.path)
  );
  return {
    values: { ...values },
    json,
    keyValues,
    invalidKeyValues,
    present: new Set(Object.keys(values)),
  };
}

export function updateParamDraftValue(draft: ParamDraft, path: string, value: unknown): ParamDraft {
  return {
    ...draft,
    values: { ...draft.values, [path]: value },
    present: new Set(draft.present).add(path),
  };
}

export function updateJsonDraft(draft: ParamDraft, path: string, value: string): ParamDraft {
  return {
    ...draft,
    json: { ...draft.json, [path]: value },
    present: new Set(draft.present).add(path),
  };
}

export function updateKeyValueDraft(
  draft: ParamDraft,
  path: string,
  rows: readonly KeyValueDraftRow[]
): ParamDraft {
  const invalidKeyValues = new Set(draft.invalidKeyValues);
  invalidKeyValues.delete(path);
  return {
    ...draft,
    keyValues: { ...draft.keyValues, [path]: rows },
    invalidKeyValues,
    present: new Set(draft.present).add(path),
  };
}

export function materializeParamDraft(draft: ParamDraft, fields: ParamFieldSpec[]): ParamValues {
  return fields.reduce<ParamValues>(
    (values, field) => {
      if (!draft.present.has(field.path)) return values;
      if (field.widget === PARAM_WIDGET.JSON_EDITOR) {
        return { ...values, [field.path]: parseJsonDraft(draft, field.path) };
      }
      if (field.widget === PARAM_WIDGET.KEY_VALUE) {
        if (draft.invalidKeyValues.has(field.path)) {
          throw new ParamDraftError('invalid_key_value', field.path);
        }
        return { ...values, [field.path]: materializeKeyValues(draft, field.path) };
      }
      return values;
    },
    { ...draft.values }
  );
}

function createKeyValueRows(
  record: Record<string, string>,
  createId: () => string
): readonly KeyValueDraftRow[] {
  return Object.entries(record).map(([key, item]) => ({
    id: createId(),
    key,
    value: item,
  }));
}

export function materializeKeyValueRows(
  rows: readonly KeyValueDraftRow[],
  path: string
): Record<string, string> {
  const result: Record<string, string> = {};
  rows.forEach((row) => {
    if (!row.key.trim()) throw new ParamDraftError('empty_key', path);
    if (Object.hasOwn(result, row.key)) throw new ParamDraftError('duplicate_key', path);
    result[row.key] = row.value;
  });
  return result;
}

export function isParamFieldDisabled(field: ParamFieldSpec, values: ParamValues): boolean {
  const condition = field.disabled_when;
  return condition ? condition.values.includes(values[condition.path]) : false;
}

function materializeKeyValues(draft: ParamDraft, path: string) {
  return materializeKeyValueRows(draft.keyValues[path] ?? [], path);
}

function parseJsonDraft(draft: ParamDraft, path: string): unknown {
  try {
    return JSON.parse(draft.json[path] ?? '');
  } catch {
    throw new ParamDraftError('invalid_json', path);
  }
}

function createInitialKeyValueDraft(
  field: ParamFieldSpec,
  values: ParamValues,
  createId: () => string
): InitialKeyValueDraft {
  const record = asStringRecord(values[field.path]);
  const invalid = Object.hasOwn(values, field.path) && record === null;
  return { path: field.path, rows: record ? createKeyValueRows(record, createId) : [], invalid };
}

function asStringRecord(value: unknown): Record<string, string> | null {
  if (typeof value !== 'object' || value === null || Array.isArray(value)) return null;
  const entries = Object.entries(value);
  if (!entries.every(([, item]) => typeof item === 'string')) return null;
  return Object.fromEntries(entries) as Record<string, string>;
}
