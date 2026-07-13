import type { ParamDraftError } from './param-draft';
import type { ParamSchema, ParamFieldSpec } from 'src/entities/scheduler';

import { it, expect, describe } from 'vitest';

import { PARAM_WIDGET } from 'src/entities/scheduler';

import { compileParamSchema } from './param-schema';
import {
  updateJsonDraft,
  createParamDraft,
  updateKeyValueDraft,
  isParamFieldDisabled,
  materializeParamDraft,
  materializeKeyValueRows,
} from './param-draft';

const FIELDS: ParamFieldSpec[] = [
  paramField('headers', PARAM_WIDGET.KEY_VALUE),
  paramField('body', PARAM_WIDGET.JSON_EDITOR),
  paramField('retries', PARAM_WIDGET.NUMBER),
  paramField('enabled', PARAM_WIDGET.SWITCH),
];

function paramField(path: string, widget: ParamFieldSpec['widget']): ParamFieldSpec {
  return {
    path,
    label: path,
    widget,
    placeholder: null,
    help: null,
    options: [],
    disabled_when: null,
  };
}

describe('scheduler parameter drafts', () => {
  it('assigns stable IDs and materializes typed values', () => {
    let sequence = 0;
    const draft = createParamDraft(
      { headers: { Accept: 'json' }, body: { ok: true }, retries: 2, enabled: false },
      FIELDS,
      () => `row-${sequence++}`
    );

    expect(draft.keyValues.headers?.[0]?.id).toBe('row-0');
    expect(materializeParamDraft(draft, FIELDS)).toEqual({
      headers: { Accept: 'json' },
      body: { ok: true },
      retries: 2,
      enabled: false,
    });
  });

  it.each([
    [[{ id: '1', key: '', value: 'x' }], 'empty_key'],
    [
      [
        { id: '1', key: 'x', value: '1' },
        { id: '2', key: 'x', value: '2' },
      ],
      'duplicate_key',
    ],
  ] as const)('rejects invalid key-value rows', (rows, code) => {
    expect(() => materializeKeyValueRows(rows, 'headers')).toThrowError(
      expect.objectContaining<Partial<ParamDraftError>>({ code, path: 'headers' })
    );
  });

  it('keeps invalid JSON as a draft and fails explicitly on materialization', () => {
    const draft = createParamDraft({ body: {} }, FIELDS, () => 'row');
    const invalid = updateJsonDraft(draft, 'body', '{');

    expect(invalid.json.body).toBe('{');
    expect(() => materializeParamDraft(invalid, FIELDS)).toThrowError(
      expect.objectContaining<Partial<ParamDraftError>>({ code: 'invalid_json', path: 'body' })
    );
  });
});

describe('optional and conditional parameters', () => {
  it('omits untouched optional structured fields', () => {
    const optionalFields = [
      paramField('body', PARAM_WIDGET.JSON_EDITOR),
      paramField('headers', PARAM_WIDGET.KEY_VALUE),
    ];
    const draft = createParamDraft({}, optionalFields, () => 'row');

    expect(materializeParamDraft(draft, optionalFields)).toEqual({});
    expect(materializeParamDraft(updateJsonDraft(draft, 'body', '{}'), optionalFields)).toEqual({
      body: {},
    });
  });

  it('applies disabled_when to typed boolean values', () => {
    const field = {
      ...paramField('body', PARAM_WIDGET.JSON_EDITOR),
      disabled_when: { path: 'enabled', values: [false] },
    };

    expect(isParamFieldDisabled(field, { enabled: false })).toBe(true);
    expect(isParamFieldDisabled(field, { enabled: true })).toBe(false);
  });
});

describe('invalid key-value initial values', () => {
  it.each(['raw', null, [], { Authorization: 123 }])(
    'preserves and rejects non-string record value %#',
    (initialValue) => {
      const draft = createParamDraft({ headers: initialValue }, FIELDS, () => 'row');

      expect(draft.values.headers).toBe(initialValue);
      expect(draft.invalidKeyValues.has('headers')).toBe(true);
      expect(() => materializeParamDraft(draft, FIELDS)).toThrowError(
        expect.objectContaining<Partial<ParamDraftError>>({
          code: 'invalid_key_value',
          path: 'headers',
        })
      );
      expect(draft.values.headers).toBe(initialValue);
    }
  );

  it('clears the initial error only after the user replaces the key-value draft', () => {
    const draft = createParamDraft({ headers: 'raw' }, FIELDS, () => 'row');
    const repaired = updateKeyValueDraft(draft, 'headers', [
      { id: 'row', key: 'Accept', value: 'application/json' },
    ]);

    expect(draft.invalidKeyValues.has('headers')).toBe(true);
    expect(repaired.invalidKeyValues.has('headers')).toBe(false);
    expect(materializeParamDraft(repaired, FIELDS).headers).toEqual({
      Accept: 'application/json',
    });
  });
});

describe('parameter schema compiler', () => {
  const schema: ParamSchema = {
    type: 'object',
    required: ['retries', 'enabled'],
    additional_properties: false,
    properties: {
      retries: { type: 'number', min: 1, max: 3 },
      enabled: { type: 'boolean' },
    },
  };

  it('accepts typed number and boolean values', () => {
    expect(compileParamSchema(schema).parse({ retries: 2, enabled: true })).toEqual({
      retries: 2,
      enabled: true,
    });
  });

  it.each([
    { retries: '2', enabled: true },
    { retries: 4, enabled: true },
    { retries: 2, enabled: true, extra: 'no' },
  ])('rejects invalid typed values and additional properties', (value) => {
    expect(() => compileParamSchema(schema).parse(value)).toThrow();
  });
});
