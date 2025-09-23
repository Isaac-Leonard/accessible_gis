import {
  FeatureInfo,
  Field,
  FieldType,
  VectorScreenData,
  VectorScreenMetadata,
} from "../bindings";
import { GeometryViewer } from "../geometry";
import { OptionPicker } from "../option-picker";
import { FeatureCreator } from "../feature-creator";
import { useState } from "preact/hooks";
import { client } from "../api";
import { Dialog, useDialog } from "../dialog";
import { ReprojectionDialog } from "../reprojection-dialog";
import { FeaturePicker } from "../feature-picker";
import { FeatureCoppierDialog } from "../feature-copier";
import { LayerSimplifierDialog } from "../layer_simplifier";
import { GdalMetadataViewer } from "../raster-navigator";

type VectorLayerProp = {
  layer: VectorScreenData;
};

export const VectorNavigator = ({ layer }: VectorLayerProp) => {
  return (
    <div>
      <VectorMetadataDialog metadata={layer.metadata} />
      <ReprojectionDialog />
      <FeatureCoppierDialog layer={layer} />
      <LayerSimplifierDialog />
      <button
        onClick={(_) =>
          client
            .getLandformDescription()
            .then((description) => alert(description))
        }
      >
        View Landform Description
      </button>
      {layer.editable ? <DatasetEditor layer={layer} /> : <EditDatasetButton />}
      {layer.display ? (
        <>
          <div>Displayed</div>
          <button onClick={client.focusDataset}>Focus Layer</button>
          <button
            onClick={() => client.toggleLabels()}
            role="switch"
            aria-checked={layer.use_labels}
          >
            Toggle Auto Labels
          </button>
          <button
            onClick={() => client.toggleAnnounceLeaving()}
            role="switch"
            aria-checked={layer.announce_leaving}
          >
            Toggle announcements when leaving polygons
          </button>
          <button
            onClick={() => client.toggleAnnounceGeometryTypes()}
            role="switch"
            aria-checked={layer.announce_geometry_type}
          >
            Toggle announcing types of geometries
          </button>
        </>
      ) : (
        <button onClick={client.setDisplayVector}>Show on screen</button>
      )}
      <NameFieldPicker layer={layer} />
      <FeaturePicker layer={layer} />
      <FeatureSorter layer={layer} />
      <FeatureViewer layer={layer} />
    </div>
  );
};

export type FieldsTableProps = {
  fields: Field[];
  preferedDisplayField: string | null;
};

function FieldsTable({ fields, preferedDisplayField }: FieldsTableProps) {
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

type CurrentFeatureViewerProps = {
  info: FeatureInfo;
  layer: VectorScreenData;
};

const CurrentFeatureViewer = ({ info, layer }: CurrentFeatureViewerProps) => {
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
        {info.geometry !== null ? (
          <GeometryViewer geometry={info.geometry} srs={layer.srs} />
        ) : (
          <div>No Geometry</div>
        )}
      </Dialog>
      <Dialog
        modal={true}
        open={fieldsOpen}
        setOpen={setFieldsOpen}
        openText="Open fields table"
      >
        <FieldsTable
          fields={info.fields}
          preferedDisplayField={layer.prefered_display_field}
        />
      </Dialog>
    </div>
  );
};

const FeatureViewer = ({ layer }: VectorLayerProp) => {
  return layer.feature !== null ? (
    <CurrentFeatureViewer info={layer.feature} layer={layer} />
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

const FeatureSorter = ({ layer }: NameFieldPickerProps) => {
  const { name_field } = layer;
  const field_names = layer.field_schema.map((field) => field.name);
  const options = ["Default", "Field", "Area"] as const;

  const setOption = (option: (typeof options)[number]) => {
    if (option === "Field") {
      client.sortFeaturesBy({ option, settings: name_field });
    } else {
      client.sortFeaturesBy({ option });
    }
  };

  return (
    <div>
      <OptionPicker
        prompt="Sort features by"
        options={options}
        selectedOption={layer.sort_features_by.option}
        setOption={setOption}
        emptyText="This should not be empty"
      />
      {layer.sort_features_by.option === "Field" ? (
        <OptionPicker
          options={field_names}
          selectedOption={layer.sort_features_by.settings}
          setOption={(field) => {
            client.sortFeaturesBy({ option: "Field", settings: field });
          }}
          emptyText=" This layer has no fields"
          prompt="Set name field"
        />
      ) : (
        ""
      )}
    </div>
  );
};

const VectorMetadataDialog = ({
  metadata,
}: {
  metadata: VectorScreenMetadata;
}) => {
  const { open, setOpen } = useDialog();
  return (
    <Dialog
      openText="Layer Metadata"
      modal={true}
      open={open}
      setOpen={setOpen}
    >
      <h3>projection</h3>
      <div>srs: {metadata.srs}</div>
      <GdalMetadataViewer metadata={metadata.other} />
    </Dialog>
  );
};
