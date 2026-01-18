
export function canonicalJson(value: unknown): string {
  return JSON.stringify(value, Object.keys(value as object).sort());
}
