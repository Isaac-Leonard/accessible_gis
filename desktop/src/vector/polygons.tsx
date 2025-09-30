import { message } from "@tauri-apps/plugin-dialog";
import { PointsTableView } from "./points-table";
import { Polygon, commands } from "../bindings";
import { useEffect, useState } from "preact/hooks";
import { useSignal } from "@preact/signals";

function PolygonTableView({ polygon }: { polygon: Polygon }) {
  return (
    <div>
      Exterior points: <PointsTableView line={polygon.exterior} />;
    </div>
  );
}

export const PolygonViewer = ({ polygon }: { polygon: Polygon }) => {
  return (
    <div>
      Polygon:
      <PolygonTableView polygon={polygon} />
    </div>
  );
};
