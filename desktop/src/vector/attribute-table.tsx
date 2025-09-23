import { client } from "../api";
import { Field } from "../bindings";

export type FieldsTableProps = {
  fields: Field[];
  preferedDisplayField: string | null;
};

export function FieldsTable({
  fields,
  preferedDisplayField,
}: FieldsTableProps) {
  return (
    <table>
      <thead>
        <tr>
          <th>Field</th>
          <th>Value</th>
          <th>Options</th>
        </tr>
      </thead>
      <tbody>
        {fields.map((field) => (
          <tr key={field.name}>
            <td>{field.name}</td>
            <td>
              <FieldValueViewer field={field} />
            </td>
            <td>
              {preferedDisplayField === field.name ? (
                <span>Displayed</span>
              ) : (
                <button
                  onClick={() => client.setPreferedDisplayField(field.name)}
                >
                  Use as prefered display field
                </button>
              )}
            </td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}

function FieldValueViewer({ field }: { field: Field }) {
  switch (field.type) {
    case "Integer":
      return <span>Integer: {field.value}</span>;
    case "Real":
      return <span>Real: {field.value}</span>;
    case "Integer64":
      return <span>64 bit integer: {field.value}</span>;
    case "DateTime":
      return <span>Datetime: {field.value}</span>;
    case "Date":
      return <span>Date: {field.value}</span>;
    case "String":
      return <span>String: {field.value}</span>;
    case "StringList":
      const quotedStrings = field.value.map((str) => JSON.stringify(str));
      const first3Strings = `[${quotedStrings.slice(0, 3).join(",")}]`;
      if (first3Strings.length < 200) {
        return <span>String list: {first3Strings}</span>;
      } else {
        return (
          <span>
            <select>
              String list:{" "}
              {quotedStrings.map((str) => (
                <option key={str}>{str}</option>
              ))}
            </select>
          </span>
        );
      }
    case "IntegerList":
      if (field.value.length < 3) {
        return (
          <span> Integer list: [{field.value.slice(0, 3).join(", ")}]</span>
        );
      } else {
        return (
          <span>
            Integer list:
            <select>
              {field.value.map((val) => (
                <option key={val}>{val}</option>
              ))}
            </select>
          </span>
        );
      }
    case "Integer64List":
      if (field.value.length < 3) {
        return (
          <span>
            {" "}
            64 bit integer list: [{field.value.slice(0, 3).join(", ")}]
          </span>
        );
      } else {
        return (
          <span>
            64 bit integer list:
            <select>
              {field.value.map((val) => (
                <option key={val}>{val}</option>
              ))}
            </select>
          </span>
        );
      }
    case "RealList":
      if (field.value.length < 3) {
        return <span> Real list: [{field.value.slice(0, 3).join(", ")}]</span>;
      } else {
        return (
          <span>
            Real list:
            <select>
              {field.value.map((val) => (
                <option key={val}>{val}</option>
              ))}
            </select>
          </span>
        );
      }
    case "None":
      return <span>Empty</span>;
    default:
      return <span>Unknown</span>;
  }
}
