import { message } from "@tauri-apps/api/dialog";
import { suspend } from "suspend-react";
import { LineString, describeLine } from "./bindings";
import { Suspense, useDeferredValue, useState } from "react";
import { PointsTableView } from "./points-table";

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
        <Suspense fallback=" Loading">
          <LineStringDescription line={line} srs={srs} />
        </Suspense>
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
  let [distance, setDistance] = useState("20000");
  let [towns, setTowns] = useState("20");
  towns = useDeferredValue(towns);
  distance = useDeferredValue(distance);
  const description = useDeferredValue(
    suspend(
      () =>
        describeLine(line, srs, Number(distance), Number(towns)).catch((e) => {
          message(e as string);
          throw e;
        }),
      [line, distance, srs, towns]
    )
  );
  return (
    <div>
      <div>{JSON.stringify(description)}</div>
      <div>
        <label>
          Distance to towns:
          <input
            type="number"
            value={distance}
            onChange={(e) => setDistance(e.target.value)}
          />
        </label>
        <label>
          Max number of towns:
          <input
            type="number"
            value={towns}
            onChange={(e) => setTowns(e.target.value)}
          />
        </label>
      </div>
    </div>
  );
};
