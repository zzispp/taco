import type { ParamSchema } from 'src/entities/scheduler';

import * as z from 'zod';

type Compiler = (schema: ParamSchema) => z.ZodTypeAny;

const compilers: Record<ParamSchema['type'], Compiler> = {
  object: compileObject,
  string: compileString,
  number: compileNumber,
  boolean: () => z.boolean(),
  record: compileRecord,
  array: compileArray,
};

export function compileParamSchema(schema: ParamSchema): z.ZodTypeAny {
  const compiler = compilers[schema.type];
  if (!compiler) throw new Error(`unsupported param schema type: ${schema.type}`);
  return compiler(schema);
}

function compileObject(schema: ParamSchema) {
  if (schema.type !== 'object')
    throw new Error('object schema compiler received non-object schema');
  const shape = Object.fromEntries(
    Object.entries(schema.properties).map(([key, value]) => {
      const field = compileParamSchema(value);
      return [key, schema.required.includes(key) ? field : field.optional()];
    })
  );
  const objectSchema = z.object(shape);
  return schema.additional_properties ? objectSchema.passthrough() : objectSchema.strict();
}

function compileString(schema: ParamSchema) {
  if (schema.type !== 'string')
    throw new Error('string schema compiler received non-string schema');
  let result = z.string();
  if (schema.pattern) result = result.regex(new RegExp(schema.pattern));
  if (schema.enum_values.length > 0) {
    result = result.refine((value) => schema.enum_values.includes(value));
  }
  return result;
}

function compileNumber(schema: ParamSchema) {
  if (schema.type !== 'number')
    throw new Error('number schema compiler received non-number schema');
  let result = z.number();
  if (typeof schema.min === 'number') result = result.min(schema.min);
  if (typeof schema.max === 'number') result = result.max(schema.max);
  return result;
}

function compileRecord(schema: ParamSchema) {
  if (schema.type !== 'record')
    throw new Error('record schema compiler received non-record schema');
  return z.record(compileRecordKey(schema.key), compileParamSchema(schema.value));
}

function compileArray(schema: ParamSchema) {
  if (schema.type !== 'array') throw new Error('array schema compiler received non-array schema');
  return z.array(compileParamSchema(schema.items));
}

function compileRecordKey(schema: ParamSchema) {
  if (schema.type !== 'string')
    throw new Error(`unsupported record key schema type: ${schema.type}`);
  return compileString(schema);
}
