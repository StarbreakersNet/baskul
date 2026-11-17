export interface ChoiceOption<T extends string> {
  value: T;
  label: string;
  disabled?: boolean;
}
