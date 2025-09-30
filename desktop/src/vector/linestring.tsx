import { PointsTableView } from "./points-table";
import { LineString } from "../bindings";

export const LineStringView = ({ line }: { line: LineString }) => {
  return (
    <div>
      Linestring:
      <PointsTableView line={line} />
    </div>
  );
};
