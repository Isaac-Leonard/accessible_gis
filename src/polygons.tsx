import { message } from "@tauri-apps/plugin-dialog";
import { PointsTableView } from "./points-table";
import { Polygon, commands } from "./bindings";
import { useEffect, useState } from "preact/hooks";
import { useSignal } from "@preact/signals";

function PolygonDescription({
  polygon: { exterior, interior },
}: {
  polygon: Polygon;
}) {
  const description = useSignal<null | string>(null);
  useEffect(() => {
    commands
      .describePolygon({ exterior, interior })
      .then((value) => {
        description.value = value;
      })
      .catch((e) => {
        message(e as string);
        throw e;
      });
  }, [exterior, interior]);
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
        <PolygonDescription polygon={polygon} />
      )}
    </div>
  );
};
