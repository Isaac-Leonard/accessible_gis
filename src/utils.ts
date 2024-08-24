import { TargetedEvent } from "preact/compat";

export type SetterName<S extends string> = `set${Capitalize<S>}`;

export const setterName = <S extends string>(name: S): SetterName<S> => {
  const [first, ...rest] = name;
  return `set${first.toUpperCase()}${rest}` as `set${Capitalize<S>}`;
};

export type GetSet<P extends string, T extends {}, K extends keyof T> = {
  [k in P]: T[K];
} & {
  [r in ReturnType<typeof setterName<P>>]: (val: T[K]) => void;
};

export const getterSetterFactory = <T extends {}>(
  value: T,
  setValue: (value: T) => void
) => ({
  onChange: <K extends keyof T>(name: K) => ({
    value: value[name],
    onChange: (e: TargetedEvent<HTMLInputElement, Event>) =>
      setValue({ ...value, [name]: e.currentTarget.value }),
  }),
  getSet: <K extends keyof T, P extends string>(
    name: K,
    prop: P
  ): GetSet<P, T, K> =>
    ({
      [prop]: value[name],
      [setterName(prop)]: (propVal: T[K]) =>
        setValue({ ...value, [name]: propVal }),
    } as GetSet<P, T, K>),
});
