type UnionToIntersection<T> = (T extends unknown ? (value: T) => void : never) extends (
  value: infer TIntersection
) => void
  ? TIntersection
  : never;

export function mergeAdminResources<TResources extends readonly object[]>(
  ...resources: TResources
): UnionToIntersection<TResources[number]> {
  return resources.reduce<Record<string, unknown>>((merged, resource) => {
    for (const key of Object.keys(resource)) {
      if (Object.hasOwn(merged, key)) throw new Error(`Duplicate admin resource key: ${key}`);
    }
    return { ...merged, ...resource };
  }, {}) as UnionToIntersection<TResources[number]>;
}
