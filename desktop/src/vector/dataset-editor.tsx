import { useState } from "preact/hooks";
import { client } from "../api";
import { Dialog, useDialog } from "../dialog";
import { FieldType, VectorScreenData } from "../bindings";
import { OptionPicker } from "../option-picker";
import { FeatureCreator } from "./feature-creator";

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
  const { open, setOpen, innerRef } = useDialog<HTMLInputElement>();
  const [name, setName] = useState("");
  const [fieldType, setFieldType] = useState<FieldType>(fieldTypes[0]);
  return (
    <Dialog
      modal={true}
      open={open}
      setOpen={setOpen}
      openText="Add field to schema"
    >
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
    </Dialog>
  );
};

export const DatasetEditor = ({ layer }: { layer: VectorScreenData }) => {
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

export const EditDatasetButton = () => {
  return <button onClick={client.editDataset}>Edit</button>;
};
