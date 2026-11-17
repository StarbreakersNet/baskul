export function slotsExcept<T extends object>(slots: T, ...names: string[]) {
  return Object.fromEntries(Object.entries(slots).filter(([name]) => !names.includes(name)));
}
