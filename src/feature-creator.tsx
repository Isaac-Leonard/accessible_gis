import { useState } from "preact/hooks";
import {
  FeatureInfo,
  Field,
  FieldSchema,
  FieldType,
  Geometry,
  GeometryCollection,
  Line,
  LineString,
  MultiLineString,
  MultiPoint,
  MultiPolygon,
  Point,
  Polygon,
} from "./bindings";
import { OptionPicker } from "./option-picker";
import { Ref } from "preact";

export const FeatureCreator = ({
  setFeature,
  schema,
  focusRef,
}: {
  focusRef: Ref<any>;
  setFeature: (f: FeatureInfo) => void;
  schema: FieldSchema[];
}) => {
  const [geometry, setGeometry] = useState<Geometry>({
    type: "Point",
    x: 0,
    y: 0,
  });
  const [fields, setFields] = useState<Field[]>(() =>
    schema.map((schema) => ({
      name: schema.name,
      ...defaultTypeFromSchema(schema.field_type!),
    }))
  );
  return (
    <div ref={focusRef} tabIndex={0}>
      <GeometryEditor geometry={geometry} setGeometry={setGeometry} />{" "}
      <div>
        Fields:{" "}
        {fields.map((field, i) => (
          <FieldEditor
            key={field.name}
            field={field}
            setField={(field) => {
              fields.splice(i, 1, field);
              setFields([...fields]);
            }}
          />
        ))}
      </div>
      <button onClick={() => setFeature({ geometry, fields, fid: null })}>
        Create
      </button>
    </div>
  );
};

const FieldEditor = ({
  field,
  setField,
}: {
  field: Field;
  setField: (field: Field) => void;
}) => {
  return (
    <div>
      {field.name}
      <FieldValueEditor field={field} setField={setField} />
    </div>
  );
};

const FieldValueEditor = ({
  field,
  setField,
}: {
  field: Field;
  setField: (field: Field) => void;
}) => {
  switch (field.type) {
    case "Integer":
    case "Integer64":
      return (
        <input
          pattern="\d*"
          value={field.value}
          onChange={(e) =>
            setField({ ...field, value: Number(e.currentTarget.value) })
          }
        />
      );
    case "String":
    case "Date":
    case "DateTime":
      return (
        <input
          value={field.value}
          onChange={(e) => setField({ ...field, value: e.currentTarget.value })}
        />
      );
    case "Real":
      return (
        <input
          type="number"
          value={field.value}
          onChange={(e) =>
            setField({ ...field, value: Number(e.currentTarget.value) })
          }
        />
      );
    default:
      return (
        <div>Cannot currently create list fields or got unknown field type</div>
      );
  }
};

// Helper type to distribute Omit
type DistributeOmit<T, K extends keyof any> = T extends any
  ? Omit<T, K>
  : never;

export type FieldValue = DistributeOmit<Field, "name">;

const defaultTypeFromSchema = (schema: FieldType): FieldValue => {
  switch (schema) {
    case "OFTInteger":
      return { type: "Integer", value: 0 };
    case "OFTReal":
      return { type: "Real", value: 0 };
    case "OFTString":
      return { type: "String", value: "" };
    case "OFTRealList":
      return { type: "RealList", value: [] };
    case "OFTStringList":
      return { type: "StringList", value: [] };
    case "OFTInteger64":
      return { type: "Integer64", value: 0 };
    case "OFTIntegerList":
      return { type: "IntegerList", value: [] };
    case "OFTInteger64List":
      return { type: "Integer64List", value: [] };
    default:
      throw "Cannot handle type " + schema;
  }
};

const geometryTypes = [
  "Point",
  "Line",
  "LineString",
  "Polygon",
  "MultiPoint",
  "MultiLineString",
  "MultiPolygon",
  "GeometryCollection",
] as const;

const GeometryEditor = ({
  geometry,
  setGeometry,
}: {
  geometry: Geometry;
  setGeometry: (geometry: Geometry) => void;
}) => {
  return (
    <div>
      <OptionPicker
        options={geometryTypes}
        selectedOption={geometry.type}
        setOption={(gtype) => setGeometry(defaultGeometryForType(gtype as any))}
        prompt="Kind of Geometry"
        emptyText="This should not be empty geometries"
      />
      <GeometryValueEditor geometry={geometry} setGeometry={setGeometry} />
    </div>
  );
};

const defaultGeometryForType = (gtype: Geometry["type"]): Geometry => {
  switch (gtype) {
    case "Point":
      return { type: gtype, x: 0, y: 0 };
    case "LineString":
      return { type: gtype, points: [] };
    case "Polygon":
      return { type: gtype, exterior: { points: [] }, interior: [] };
    case "MultiPoint":
      return { type: gtype, points: [] };
    case "MultiLineString":
      return { type: gtype, lines: [] };
    case "MultiPolygon":
      return { type: gtype, polygons: [] };
    case "Line":
      return { type: gtype, start: { x: 0, y: 0 }, end: { x: 0, y: 0 } };
    case "GeometryCollection":
      return { type: gtype, geometries: [] };
  }
};

const GeometryValueEditor = ({
  geometry,
  setGeometry,
}: {
  geometry: Geometry;
  setGeometry: (geometry: Geometry) => void;
}) => {
  switch (geometry.type) {
    case "Point":
      return (
        <PointEditor
          point={geometry}
          setPoint={(p) => setGeometry({ type: geometry.type, ...p })}
        />
      );
    case "Line":
      return (
        <LineEditor
          line={geometry}
          setLine={(line) => setGeometry({ type: geometry.type, ...line })}
        />
      );
    case "LineString":
      return (
        <LineStringEditor
          line={geometry}
          setLine={(line) => setGeometry({ type: geometry.type, ...line })}
        />
      );
    case "Polygon":
      return (
        <PolygonEditor
          polygon={geometry}
          setPolygon={(polygon) =>
            setGeometry({ type: geometry.type, ...polygon })
          }
        />
      );
    case "MultiPoint":
      return (
        <MultiPointEditor
          multipoint={geometry}
          setMultiPoint={(multipoint) =>
            setGeometry({ type: geometry.type, ...multipoint })
          }
        />
      );
    case "MultiLineString":
      return (
        <MultiLineStringEditor
          multiline={geometry}
          setMultiLine={(multiline) =>
            setGeometry({ type: geometry.type, ...multiline })
          }
        />
      );
    case "MultiPolygon":
      return (
        <MultiPolygonEditor
          multipolygon={geometry}
          setMultiPolygon={(multipolygon) =>
            setGeometry({ type: geometry.type, ...multipolygon })
          }
        />
      );
    case "GeometryCollection":
      return (
        <GeometryCollectionEditor
          collection={geometry}
          setCollection={(collection) =>
            setGeometry({ type: geometry.type, ...collection })
          }
        />
      );
    default:
      return (
        <div>
          Unsupported or unknown geometry type {(geometry as Geometry).type}
        </div>
      );
  }
};

const PointEditor = ({
  point: { x, y },
  setPoint,
}: {
  point: Point;
  setPoint: (point: Point) => void;
}) => {
  return (
    <div>
      <label>
        x:
        <input
          type="number"
          value={x}
          onChange={(e) => setPoint({ x: Number(e.currentTarget.value), y })}
        />
      </label>
      <label>
        y:
        <input
          type="number"
          value={y}
          onChange={(e) => setPoint({ x, y: Number(e.currentTarget.value) })}
        />
      </label>
    </div>
  );
};

const LineEditor = ({
  line: { start, end },
  setLine,
}: {
  line: Line;
  setLine: (line: Line) => void;
}) => {
  return (
    <div>
      Start:
      <PointEditor
        point={start}
        setPoint={(start) => setLine({ start, end })}
      />
      end:
      <PointEditor point={end} setPoint={(end) => setLine({ start, end })} />
    </div>
  );
};

const LineStringEditor = ({
  line,
  setLine,
}: {
  line: LineString;
  setLine: (line: LineString) => void;
}) => {
  return (
    <div>
      Points in line:
      <ol>
        {line.points.map((point, i) => (
          <li key={i}>
            <PointEditor
              point={point}
              setPoint={(point) => {
                line.points.splice(i, 1, point);
                setLine({ points: [...line.points] });
              }}
            />
            <button
              onClick={() => {
                line.points.splice(i, 1);
                setLine({ points: [...line.points] });
              }}
            >
              Remove this point
            </button>{" "}
          </li>
        ))}
      </ol>
      <button
        onClick={() =>
          setLine({
            points: [...line.points, { x: 0, y: 0 }],
          })
        }
      >
        Add point to line
      </button>
    </div>
  );
};

const PolygonEditor = ({
  polygon: { exterior, interior },
  setPolygon,
}: {
  polygon: Polygon;
  setPolygon: (polygon: Polygon) => void;
}) => {
  return (
    <div>
      Exterior:
      <LineStringEditor
        line={exterior}
        setLine={(exterior) => setPolygon({ exterior, interior })}
      />
      <div>
        Interior:
        <ol>
          {interior.map((line, i) => (
            <li key={i}>
              <LineStringEditor
                line={line}
                setLine={(interiorLine) => {
                  interior.splice(i, 1, interiorLine);
                  setPolygon({ exterior, interior: [...interior] });
                }}
              />
              <button
                onClick={() => {
                  interior.splice(i, 1);
                  setPolygon({ exterior, interior: [...interior] });
                }}
              >
                Remove this interior line
              </button>{" "}
            </li>
          ))}
        </ol>
        <button
          onClick={() =>
            setPolygon({
              exterior,
              interior: [...interior, { points: [] }],
            })
          }
        >
          Add interior line
        </button>
      </div>
    </div>
  );
};

const MultiPointEditor = ({
  multipoint: { points },
  setMultiPoint,
}: {
  multipoint: MultiPoint;
  setMultiPoint: (multipoint: MultiPoint) => void;
}) => {
  return (
    <div>
      Points in MultiPoint:
      <ol>
        {points.map((point, i) => (
          <li key={i}>
            <PointEditor
              point={point}
              setPoint={(point) => {
                points.splice(i, 1, point);
                setMultiPoint({ points: [...points] });
              }}
            />
            <button
              onClick={() => {
                points.splice(i, 1);
                setMultiPoint({ points: [...points] });
              }}
            >
              Remove this point
            </button>{" "}
          </li>
        ))}
      </ol>
      <button
        onClick={() =>
          setMultiPoint({
            points: [...points, { x: 0, y: 0 }],
          })
        }
      >
        Add point to MultiPoint
      </button>
    </div>
  );
};

const MultiLineStringEditor = ({
  multiline: { lines },
  setMultiLine,
}: {
  multiline: MultiLineString;
  setMultiLine: (multiline: MultiLineString) => void;
}) => {
  return (
    <div>
      Lines in MultiLineString:
      <ol>
        {lines.map((line, i) => (
          <li key={i}>
            <LineStringEditor
              line={line}
              setLine={(line) => {
                lines.splice(i, 1, line);
                setMultiLine({ lines: [...lines] });
              }}
            />
            <button
              onClick={() => {
                lines.splice(i, 1);
                setMultiLine({ lines: [...lines] });
              }}
            >
              Remove this line
            </button>{" "}
          </li>
        ))}
      </ol>
      <button
        onClick={() =>
          setMultiLine({
            lines: [{ points: [] }],
          })
        }
      >
        Add line to MultiLineString
      </button>
    </div>
  );
};

const MultiPolygonEditor = ({
  multipolygon: { polygons },
  setMultiPolygon,
}: {
  multipolygon: MultiPolygon;
  setMultiPolygon: (multipolygon: MultiPolygon) => void;
}) => {
  return (
    <div>
      Polygons in MultiPolygon:
      <ol>
        {polygons.map((polygon, i) => (
          <li key={i}>
            <PolygonEditor
              polygon={polygon}
              setPolygon={(polygon) => {
                polygons.splice(i, 1, polygon);
                setMultiPolygon({ polygons: [...polygons] });
              }}
            />
            <button
              onClick={() => {
                polygons.splice(i, 1);
                setMultiPolygon({ polygons: [...polygons] });
              }}
            >
              Remove this polygon
            </button>{" "}
          </li>
        ))}
      </ol>
      <button
        onClick={() =>
          setMultiPolygon({
            polygons: [{ exterior: { points: [] }, interior: [] }],
          })
        }
      >
        Add polygon to MultiPolygon
      </button>
    </div>
  );
};

const GeometryCollectionEditor = ({
  collection,
  setCollection,
}: {
  collection: GeometryCollection;
  setCollection: (collection: GeometryCollection) => void;
}) => {
  return (
    <div>
      Geometries in GeometryCollection:
      <ol>
        {collection.geometries.map((geometry, i) => (
          <li key={i}>
            <GeometryValueEditor
              geometry={geometry}
              setGeometry={(geometry) => {
                collection.geometries.splice(i, 1, geometry);
                setCollection({ geometries: [...collection.geometries] });
              }}
            />
            <button
              onClick={() => {
                collection.geometries.splice(i, 1);
                setCollection({ geometries: [...collection.geometries] });
              }}
            >
              Remove this geometry
            </button>{" "}
          </li>
        ))}
      </ol>
      <button
        onClick={() =>
          setCollection({
            geometries: [{ type: "Point", x: 0, y: 0 }],
          })
        }
      >
        Add geometry to GeometryCollection
      </button>
    </div>
  );
};
