import { useDeferredValue, useState } from "react";
import {
  FeatureInfo,
  Field,
  LayerDescriptor,
  getFeatureInfo,
  getFeatureNames,
  getLayerInfo,
} from "./bindings";
import { GeometryViewer } from "./geometry";
import { suspend } from "suspend-react";
import { OptionPicker } from "./option-picker";

export const VectorNavigator = ({
  layer,
}: {
  layer: Extract<LayerDescriptor, { type: "Vector" }>;
}) => {
  layer = useDeferredValue(layer);
  const overview = suspend(() => getLayerInfo(layer), [layer]);
  const defaultFieldIndex =
    overview.field_names.indexOf("name") ??
    overview.field_names.indexOf("id") ??
    overview.field_names.findIndex((x) => x.toLowerCase().includes("name"));
  const [fieldIndex, setFieldIndex] = useState(
    defaultFieldIndex === -1 ? 0 : defaultFieldIndex
  );
  const fieldName: string | null = overview.field_names[fieldIndex] ?? null;
  const featureNames = useDeferredValue(
    suspend(
      async () =>
        fieldName !== null ? getFeatureNames(fieldName, layer) : null,
      [layer, fieldName]
    )
  );
  const [featureIndex, setFeatureIndex] = useState(0);
  const feature = useDeferredValue(
    suspend(
      async () =>
        overview.features > 0
          ? await getFeatureInfo(featureIndex, layer)
          : null,
      [layer, featureIndex, overview.features]
    )
  );
  return (
    <div>
      <OptionPicker
        options={overview?.field_names}
        index={fieldIndex}
        setIndex={setFieldIndex}
        prompt="Select name fieldd"
        emptyText="No fields in this layer"
      />
      <OptionPicker
        options={featureNames?.features ?? []}
        index={featureIndex}
        setIndex={setFeatureIndex}
        prompt="Select a feature to examine"
        emptyText="This layer has no features"
      />
      {feature !== null ? (
        <FeatureViewer info={feature} srs={layer.srs} />
      ) : null}
    </div>
  );
};

function FieldsTable({ fields }: { fields: Field[] }) {
  return (
    <table>
      <thead>
        <tr>
          <th>Field</th>
          <th>Value</th>
        </tr>
      </thead>
      <tbody>
        {fields.map((field) => (
          <tr key={field.name}>
            <td>{field.name}</td>
            <td>
              <FieldValueViewer field={field} />
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

const FeatureViewer = ({
  info,
  srs,
}: {
  info: FeatureInfo;
  srs: string | null;
}) => {
  return (
    <div>
      <GeometryViewer geometry={info.geometry} srs={srs} />
      <FieldsTable fields={info.fields} />
    </div>
  );
};
