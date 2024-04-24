import { message } from "@tauri-apps/api/dialog";
import { Suspense, useState } from "react";
import { suspend } from "suspend-react";
import { PointsTableView } from "./points-table";
import { describePolygon, Polygon } from "./bindings";

function PolygonDescription({
  polygon: { exterior, interior },
}: {
  polygon: Polygon;
}) {
  const description = suspend(
    () =>
      describePolygon({ exterior, interior }).catch((e) => {
        message(e as string);
        throw e;
      }),
    [exterior, interior]
  );
  return <div>{description}</div>;
}

function PolygonTableView({ polygon }: { polygon: Polygon }) {
  return (
    <div>
      Exterior points: <PointsTableView line={polygon.exterior} />;
    </div>
  );
}

export const PolygonViewer = ({ polygon }: { polygon: Polygon }) => {
  const [tableView, setTableView] = useState(true);
  return (
    <div>
      Polygon:
      <button onClick={() => setTableView(!tableView)}>
        {tableView ? "Switch to description" : "Switch to table"}
      </button>
      {tableView ? (
        <PolygonTableView polygon={polygon} />
      ) : (
        <Suspense fallback=" Loading">
          <PolygonDescription polygon={polygon} />
        </Suspense>
      )}
    </div>
  );
};
