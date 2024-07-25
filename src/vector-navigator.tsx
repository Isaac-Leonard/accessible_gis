import { FeatureInfo, Field, FieldType, VectorScreenData } from "./bindings";
import { GeometryViewer } from "./geometry";
import { OptionPicker } from "./option-picker";
import { Drawer, useDrawer } from "./drawer";
import { FeatureCreator } from "./feature-creator";
import { useState } from "preact/hooks";
import { client } from "./api";
import { Dialog, useDialog } from "./dialog";
import { ReprojectionDialog } from "./reprojection-dialog";
import { FeaturePicker } from "./feature-picker";

type VectorLayerProp = {
  layer: VectorScreenData;
};

export const VectorNavigator = ({ layer }: VectorLayerProp) => {
  return (
    <div>
      <ReprojectionDialog />
      {layer.editable ? <DatasetEditor layer={layer} /> : <EditDatasetButton />}
      {layer.display ? (
        <div>Displayed</div>
      ) : (
        <button onClick={client.setDisplay}>Show on screen</button>
      )}
      <NameFieldPicker layer={layer} />
      <FeaturePicker layer={layer} />
      <FeatureViewer layer={layer} />
    </div>
  );
};

export type FieldsTableProps = { fields: Field[] };

function FieldsTable({ fields }: FieldsTableProps) {
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

type CurrentFeatureViewerProps = {
  info: FeatureInfo;
  srs: string | null;
};

const CurrentFeatureViewer = ({ info, srs }: CurrentFeatureViewerProps) => {
  const { open: pointsOpen, setOpen: setPointsOpen } = useDialog();
  const { open: fieldsOpen, setOpen: setFieldsOpen } = useDialog();
  return (
    <div>
      <Dialog
        modal={true}
        open={pointsOpen}
        setOpen={setPointsOpen}
        openText="Examine points"
      >
        <GeometryViewer geometry={info.geometry} srs={srs} />
      </Dialog>
      <Dialog
        modal={true}
        open={fieldsOpen}
        setOpen={setFieldsOpen}
        openText="Open fields table"
      >
        <FieldsTable fields={info.fields} />
      </Dialog>
    </div>
  );
};

const FeatureViewer = ({ layer }: VectorLayerProp) => {
  return layer.feature !== null ? (
    <CurrentFeatureViewer info={layer.feature} srs={layer.srs} />
  ) : (
    <div>This layer has no features yet, maybe create some?</div>
  );
};

type NameFieldPickerProps = {
  layer: VectorScreenData;
};

const NameFieldPicker = ({ layer }: NameFieldPickerProps) => {
  const { name_field } = layer;
  const field_names = layer.field_schema.map((field) => field.name);
  return (
    <div>
      <OptionPicker
        options={field_names}
        selectedOption={name_field}
        setOption={client.setNameField}
        emptyText=" This layer has no fields"
        prompt="Set name field"
      />
    </div>
  );
};

const fieldTypes = [
  "OFTInteger",
  "OFTIntegerList",
  "OFTReal",
  "OFTRealList",
  "OFTString",
  "OFTStringList",
  "OFTWideString",
  "OFTWideStringList",
  "OFTBinary",
  "OFTDate",
  "OFTTime",
  "OFTDateTime",
  "OFTInteger64",
  "OFTInteger64List",
] as const;

const FieldSchemaAdder = () => {
  const { open, setOpen, innerRef } = useDrawer<HTMLInputElement>();
  const [name, setName] = useState("");
  const [fieldType, setFieldType] = useState<FieldType>(fieldTypes[0]);
  return (
    <Drawer open={open} setOpen={setOpen} openText="Add field to schema">
      <div>
        <label>
          Field name
          <input
            ref={innerRef}
            value={name}
            onChange={(e) => setName(e.currentTarget.value)}
          />
        </label>
        <OptionPicker
          prompt="Field type"
          emptyText="This shouldn't be empty"
          selectedOption={fieldType}
          setOption={setFieldType as any}
          options={fieldTypes}
        />
        <button
          onClick={async () => {
            await client.addFieldToSchema(name, fieldType);
            setOpen(false);
            setName("");
          }}
        >
          Add
        </button>
      </div>
    </Drawer>
  );
};

const EditDatasetButton = () => {
  return <button onClick={client.editDataset}>Edit</button>;
};

const DatasetEditor = ({ layer }: VectorLayerProp) => {
  const { open, setOpen, innerRef } = useDialog();
  return (
    <div>
      <FieldSchemaAdder />
      <Dialog modal={true} open={open} setOpen={setOpen} openText="Add feature">
        <FeatureCreator
          schema={layer.field_schema}
          focusRef={innerRef}
          setFeature={async (feature) => {
            await client.addFeatureToLayer(feature);
            setOpen(false);
          }}
        />
      </Dialog>
    </div>
  );
};
