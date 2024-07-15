import { message } from "@tauri-apps/plugin-dialog";
import { LineDescription, LineString, commands } from "./bindings";
import { PointsTableView } from "./points-table";
import { useDrawer, Drawer } from "./drawer";
import { useEffect, useState } from "preact/hooks";

export const LineStringView = ({
  line,
  srs,
}: {
  line: LineString;
  srs: string | null;
}) => {
  const [tableView, setTableView] = useState(true);
  return (
    <div>
      Linestring:
      <button onClick={() => setTableView(!tableView)}>
        {tableView ? "Switch to description" : "Switch to table"}
      </button>
      {tableView ? (
        <PointsTableView line={line} />
      ) : (
        <LineStringDescription line={line} srs={srs} />
      )}
    </div>
  );
};

export const LineStringDescription = ({
  line,
  srs,
}: {
  line: LineString;
  srs: string | null;
}) => {
  const [distance, setDistance] = useState("20000");
  const [towns, setTowns] = useState("20");
  const [description, setDescription] = useState<LineDescription | null>(null);
  useEffect(() => {
    commands
      .describeLine(line, srs, Number(distance), Number(towns))
      .then(setDescription)
      .catch((e) => {
        message(e as string);
        throw e;
      });
  }, [line, distance, srs, towns]);
  const [descriptionParser, setDescriptionParser] = useState(
    '(description)=>{return `A ${description.type.toLowerCase()} line that intersects ${description.countries.join(", ")}`}'
  );

  const getDescription = () => {
    if (description === null) {
      return "loading";
    } else return eval(descriptionParser)(description);
  };
  const parsedDescription = () => {
    try {
      return getDescription();
    } catch (e) {
      return "An error occured evaluating the description";
    }
  };
  return (
    <div>
      <div>{parsedDescription}</div>
      <DescriptionParserEditor
        description={descriptionParser}
        setDescription={setDescriptionParser}
      />
      <div>
        <label>
          Distance to towns:
          <input
            type="number"
            value={distance}
            onChange={(e) => setDistance(e.currentTarget.value)}
          />
        </label>
        <label>
          Max number of towns:
          <input
            type="number"
            value={towns}
            onChange={(e) => setTowns(e.currentTarget.value)}
          />
        </label>
      </div>
    </div>
  );
};

const DescriptionParserEditor = ({
  description,
  setDescription,
}: {
  description: string;
  setDescription: (description: string) => void;
}) => {
  const { open, setOpen, innerRef } = useDrawer();
  const [internalDescription, setInternalDescription] = useState(description);
  return (
    <Drawer openText="Open description editor" open={open} setOpen={setOpen}>
      <label>
        Function for description
        <textarea
          ref={innerRef}
          value={internalDescription}
          onChange={(e) => setInternalDescription(e.currentTarget.value)}
        />
      </label>
      <button
        onClick={() => {
          setDescription(internalDescription);
          setOpen(false);
          console.log("hello");
        }}
      >
        Update description
      </button>
    </Drawer>
  );
};
