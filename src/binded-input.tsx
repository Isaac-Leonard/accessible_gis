import { ReadonlySignal, Signal, computed, useComputed } from "@preact/signals";
import { SetterName, setterName } from "./utils";

export type Binding<T> = Signal<T> | GetSet<T> | ComputedSetter<T>;

type GetSet<T> = { value: T; setValue: (value: T) => void };

type ComputedSetter<T> = {
  value: ReadonlySignal<T>;
  setValue: (value: T) => void;
};

type InputProps = { label: string };

type BindedInputProps = { binding: Binding<string> } & InputProps;

export const Input = ({ binding, label }: BindedInputProps) => {
  return (
    <label>
      {label ?? ""}
      {binding instanceof Signal ? (
        <SignalInput signal={binding} />
      ) : (
        <SetterInput {...binding} />
      )}
    </label>
  );
};

type SignalInputProps = { signal: Signal<string> };

export const SignalInput = ({ signal }: SignalInputProps) => {
  return (
    <input
      value={signal}
      onChange={(e) => {
        signal.value = e.currentTarget.value;
      }}
    />
  );
};

export const SetterInput = ({
  value,
  setValue,
}: ComputedSetter<string> | GetSet<string>) => {
  return (
    <input
      value={value}
      onChange={(e) => {
        setValue(e.currentTarget.value);
      }}
    />
  );
};

type BindedNumberInputProps = { binding: Binding<number> } & InputProps;

export const NumberInput = ({ binding, label }: BindedNumberInputProps) => {
  return (
    <label>
      {label ?? ""}
      {binding instanceof Signal ? (
        <SignalNumberInput signal={binding} />
      ) : (
        <SetterNumberInput {...binding} />
      )}
    </label>
  );
};

type SignalNumberInputProps = { signal: Signal<number> };

export const SignalNumberInput = ({ signal }: SignalNumberInputProps) => {
  return (
    <input
      value={signal}
      onChange={(e) => {
        signal.value = Number(e.currentTarget.value);
      }}
    />
  );
};

export const SetterNumberInput = ({
  value,
  setValue,
}: ComputedSetter<number> | GetSet<number>) => {
  return (
    <input
      value={value}
      onChange={(e) => {
        setValue(Number(e.currentTarget.value));
      }}
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

export const useBindedObjectProperties = <T extends Record<string, unknown>>(
  obj: T,
  setObj: (obj: T) => void
): {
  [K in keyof T]: {
    value: T[K];
    setValue: (value: T[K]) => void;
  };
} => {
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
