import { useState } from "preact/hooks";
import { Binding } from "./binded-input";
import { ReadonlySignal, Signal } from "@preact/signals";

export type IndexedOptionPickerProps = {
  options: string[];
  setIndex: (field: number) => void;
  index: number | null;
  prompt: string;
  emptyText: string;
};

export const IndexedOptionPicker = ({
  options,
  index,
  setIndex,
  prompt,
  emptyText,
}: IndexedOptionPickerProps) => {
  //  const [optionsPerPage, setOptionsPerPage] = useState(240);
  const [visableIndex, setVisableIndex] = useState(0);
  const visableOptions = options.slice(visableIndex, visableIndex + 240);
  if (index === null) {
    visableOptions.unshift("Select option");
  }
  if (options.length === 0) {
    return <div>{emptyText}</div>;
  } else if (options.length < 240) {
    return (
      <label>
        {prompt}:
        <select
          value={(index ?? 0).toString()}
          onChange={(e) =>
            setIndex(Number(e.currentTarget.value) - (index === null ? 1 : 0))
          }
        >
          {visableOptions.map((field, i) => (
            <option
              value={(i + visableIndex).toString()}
              key={field + (visableIndex + i)}
            >
              {field}
            </option>
          ))}
        </select>
      </label>
    );
  } else {
    return (
      <div>
        There are too many options to display
        <button
          disabled={visableIndex < 240}
          onClick={() => setVisableIndex(visableIndex - 240)}
        >
          Previous 240
        </button>
        <button
          disabled={visableIndex + 240 > options.length}
          onClick={() => setVisableIndex(visableIndex + 240)}
        >
          Next 240
        </button>
        <label>
          {prompt}:
          <select
            value={(index ?? "0").toString()}
            onChange={(e) =>
              setIndex(Number(e.currentTarget.value) - (index === null ? 1 : 0))
            }
          >
            {visableOptions.map((field, i) => (
              <option
                value={(i + visableIndex).toString()}
                key={field + (visableIndex + i)}
              >
                {field}
              </option>
            ))}
          </select>
        </label>
      </div>
    );
  }
};

export type OptionPickerProps<T extends readonly string[]> = {
  options: T;
  setOption: (field: T[number]) => void;
  selectedOption: T[number] | null;
  prompt?: string;
  emptyText: string;
};

export function OptionPicker<T extends readonly string[]>({
  options,
  selectedOption,
  setOption,
  prompt,
  emptyText,
}: OptionPickerProps<T>) {
  //  const [optionsPerPage, setOptionsPerPage] = useState(240);
  const [visableIndex, setVisableIndex] = useState(0);
  const visableOptions: ("Select option" | T[number])[] = options.slice(
    visableIndex,
    visableIndex + 240
  );
  if (selectedOption === null || selectedOption === undefined) {
    visableOptions.unshift("Select option");
  }
  if (options.length === 0) {
    return <div>{emptyText}</div>;
  } else if (options.length < 240) {
    return (
      <label>
        {typeof prompt === "string" ? `${prompt}:` : ""}
        <select
          value={selectedOption ?? undefined}
          onChange={(e) => setOption(e.currentTarget.value as T[number])}
        >
          {visableOptions.map((field, i) => (
            <option value={field} key={field + (visableIndex + i)}>
              {field}
            </option>
          ))}
        </select>
      </label>
    );
  } else {
    return (
      <div>
        There are too many options to display
        <button
          disabled={visableIndex < 240}
          onClick={() => setVisableIndex(visableIndex - 240)}
        >
          Previous 240
        </button>
        <button
          disabled={visableIndex + 240 > options.length}
          onClick={() => setVisableIndex(visableIndex + 240)}
        >
          Next 240
        </button>
        <label>
          {prompt}:
          <select
            value={selectedOption ?? undefined}
            onChange={(e) => setOption(e.currentTarget.value as T[number])}
          >
            {visableOptions.map((field, i) => (
              <option value={field} key={field + (visableIndex + i)}>
                {field}
              </option>
            ))}
          </select>
        </label>
      </div>
    );
  }
}

type BindedSelectorProps<T extends string[]> = {
  binding: Binding<T[number]>;
  prompt?: string;
};

export const bindedSelectorFactory =
  <T extends string[]>(options: T) =>
  ({ prompt, binding }: BindedSelectorProps<T>) => {
    return binding instanceof Signal ? (
      <SignalSelector options={options} signal={binding} prompt={prompt} />
    ) : (
      <GetSetSelector
        options={options}
        prompt={prompt}
        value={binding.value}
        setValue={binding.setValue}
      />
    );
  };

type SignalSelectorProps<T extends string[]> = {
  signal: Signal<T[number]>;
  options: T;
  prompt?: string;
};

export const SignalSelector = <T extends string[]>({
  signal,
  options,
  prompt,
}: SignalSelectorProps<T>) => {
  return (
    <OptionPicker
      options={options}
      // TODO: Inline this to avoid rerenders
      selectedOption={signal.value}
      setOption={(e) => {
        signal.value = e;
      }}
      prompt={prompt}
      emptyText="Somethings wrong"
    />
  );
};

type GetSetSelectorProps<T extends string[]> = {
  value: T[number] | ReadonlySignal<T[number]>;
  setValue: (value: T[number]) => void;
  options: T;
  prompt?: string;
};

export const GetSetSelector = <T extends string[]>({
  value,
  setValue,
  options,
  prompt,
}: GetSetSelectorProps<T>) => {
  return (
    <OptionPicker
      options={options}
      // TODO: Inline this to avoid rerenders
      selectedOption={typeof value === "string" ? value : value.value}
      setOption={setValue}
      prompt={prompt}
      emptyText="Somethings wrong"
    />
  );
};
