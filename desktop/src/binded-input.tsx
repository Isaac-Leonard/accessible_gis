import { ReadonlySignal, Signal, computed, useComputed } from "@preact/signals";
import { SetterName, setterName } from "./utils";
import { Ref } from "preact";
import { useState } from "preact/hooks";

export type Binding<T> = Signal<T> | GetSet<T> | ComputedSetter<T>;

export type GetSet<T> = { value: T; setValue: (value: T) => void };

type ComputedSetter<T> = {
  value: ReadonlySignal<T>;
  setValue: (value: T) => void;
};

type InputProps = { label: string; innerRef?: Ref<HTMLInputElement> };

type BindedInputProps = { binding: Binding<string> } & InputProps;

export const Input = ({ binding, label, innerRef }: BindedInputProps) => {
  return (
    <label>
      {label ?? ""}
      {binding instanceof Signal ? (
        <SignalInput signal={binding} innerRef={innerRef} />
      ) : (
        <SetterInput {...binding} />
      )}
    </label>
  );
};

type SignalInputProps = {
  signal: Signal<string>;
  innerRef?: Ref<HTMLInputElement>;
};

export const SignalInput = ({ signal, innerRef }: SignalInputProps) => {
  return (
    <input
      value={signal}
      onChange={(e) => {
        signal.value = e.currentTarget.value;
      }}
      ref={innerRef}
    />
  );
};

type SetterInputProps = (ComputedSetter<string> | GetSet<string>) & {
  innerRef?: Ref<HTMLInputElement>;
};

export const SetterInput = ({
  value,
  setValue,
  innerRef,
}: SetterInputProps) => {
  return (
    <input
      value={value}
      onChange={(e) => {
        setValue(e.currentTarget.value);
      }}
      ref={innerRef}
    />
  );
};

type BindedNumberInputProps = { binding: Binding<number> } & InputProps;

export const NumberInput = ({
  binding,
  label,
  innerRef,
}: BindedNumberInputProps) => {
  return (
    <label>
      {label ?? ""}
      {binding instanceof Signal ? (
        <SignalNumberInput signal={binding} innerRef={innerRef} />
      ) : (
        <SetterNumberInput {...binding} innerRef={innerRef} />
      )}
    </label>
  );
};

type SignalNumberInputProps = {
  signal: Signal<number>;
  innerRef?: Ref<HTMLInputElement>;
};

export const SignalNumberInput = ({
  signal,
  innerRef,
}: SignalNumberInputProps) => {
  return (
    <input
      value={signal}
      onChange={(e) => {
        signal.value = Number(e.currentTarget.value);
      }}
      ref={innerRef}
    />
  );
};

type SetterNumberInputProps = (ComputedSetter<number> | GetSet<number>) & {
  innerRef?: Ref<HTMLInputElement>;
};

export const SetterNumberInput = ({
  value,
  setValue,
  innerRef,
}: SetterNumberInputProps) => {
  return (
    <input
      value={value}
      onChange={(e) => {
        setValue(Number(e.currentTarget.value));
      }}
      ref={innerRef}
    />
  );
};

type BindedCheckboxProps = { binding: Binding<boolean> } & InputProps;

export const Checkbox = ({ binding, label }: BindedCheckboxProps) => {
  return (
    <label>
      {label ?? ""}
      {binding instanceof Signal ? (
        <SignalCheckbox signal={binding} />
      ) : (
        <SetterCheckbox value={binding.value} setValue={binding.setValue} />
      )}{" "}
    </label>
  );
};

type SignalCheckboxProps = { signal: Signal<boolean> };

export const SignalCheckbox = ({ signal }: SignalCheckboxProps) => {
  return (
    <input
      checked={signal}
      onChange={(e) => {
        signal.value = e.currentTarget.checked;
      }}
    />
  );
};

const SetterCheckbox = ({
  value,
  setValue,
}: ComputedSetter<boolean> | GetSet<boolean>) => {
  return (
    <input
      type="checkbox"
      checked={value}
      onChange={(e) => {
        setValue(e.currentTarget.checked);
      }}
    />
  );
};

export const ObjectPropertySignals = <T extends Record<string, unknown>>(
  obj: Signal<T>
): {
  [K in keyof T]: ReadonlySignal<T[K]>;
} & {
  [K in keyof T as SetterName<K & string>]: (value: T[K]) => void;
} => {
  return Object.keys(obj.peek()).reduce(
    (prev, key) => ({
      ...prev,
      [key]: computed(() => obj.value[key]),
      [setterName(key)]: (value: any) => {
        obj.value = { ...obj.value, [key]: value };
      },
    }),
    {}
  ) as any;
};

export const useObjectPropertySignals = <T extends Record<string, unknown>>(
  obj: Signal<T>
): {
  [K in keyof T]: ReadonlySignal<T[K]>;
} & {
  [K in keyof T as SetterName<K & string>]: (value: T[K]) => void;
} => {
  return Object.keys(obj.peek()).reduce(
    (prev, key) => ({
      ...prev,
      [key]: useComputed(() => obj.value[key]),
      [setterName(key)]: (value: any) => {
        obj.value = { ...obj.value, [key]: value };
      },
    }),
    {}
  ) as any;
};

export const useComputedObjectPropertySignals = <
  T extends Record<string, unknown>
>(
  obj: ReadonlySignal<T>,
  setObj: (obj: T) => void
): {
  [K in keyof T]: {
    value: ReadonlySignal<T[K]>;
    setValue: (value: T[K]) => void;
  };
} => {
  return Object.keys(obj.peek()).reduce(
    (prev, key) => ({
      ...prev,
      [key]: {
        value: useComputed(() => obj.value[key]),
        setValue: (value: any) => {
          setObj({ ...obj.value, [key]: value });
        },
      },
    }),
    {}
  ) as any;
};

export type BindedObjectProperties<T extends Record<string, unknown>> = {
  [K in keyof T]: {
    value: T[K];
    setValue: (value: T[K]) => void;
  };
};

export const useBindedObjectProperties = <T extends Record<string, unknown>>(
  obj: T,
  setObj: (obj: T) => void
): BindedObjectProperties<T> => {
  return Object.keys(obj).reduce(
    (prev, key) => ({
      ...prev,
      [key]: {
        value: obj[key],
        setValue: (value: any) => {
          setObj({ ...obj, [key]: value });
        },
      },
    }),
    {}
  ) as any;
};

export const useBindedObjectState = <T extends Record<string, unknown>>(
  obj: T
): {
  [K in keyof T]: {
    value: T[K];
    setValue: (value: T[K]) => void;
  };
} => {
  return useBindedObjectProperties(...useState(obj));
};

export const propertyGetSet = <
  T extends Record<string, unknown>,
  K extends keyof T,
  R extends T[K]
>(
  key: K,
  getSet: GetSet<T>
): GetSet<R> => ({
  value: getSet.value[key] as R,
  setValue: (value: R) => getSet.setValue({ ...getSet.value, [key]: value }),
});
