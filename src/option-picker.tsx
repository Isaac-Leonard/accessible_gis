import { useState } from "react";

export type OptionPickerProps = {
  options: string[];
  setIndex: (field: number) => void;
  index: number;
  prompt: string;
  emptyText: string;
};

export const OptionPicker = ({
  options,
  index,
  setIndex,
  prompt,
  emptyText,
}: OptionPickerProps) => {
  //  const [optionsPerPage, setOptionsPerPage] = useState(240);
  const [visableIndex, setVisableIndex] = useState(0);
  const visableOptions = options.slice(visableIndex, visableIndex + 240);
  if (options.length === 0) {
    return <div>{emptyText}</div>;
  } else if (options.length < 240) {
    return (
      <label>
        {prompt}:
        <select
          value={index.toString()}
          onChange={(e) => setIndex(Number(e.target.value))}
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
            value={index.toString()}
            onChange={(e) => setIndex(Number(e.target.value))}
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
