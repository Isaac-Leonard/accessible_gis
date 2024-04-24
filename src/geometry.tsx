import { LineStringView } from "./linestring";
import { PolygonViewer } from "./polygons";
import { Geometry, Line } from "./bindings";

export function GeometryViewer({
  geometry,
  srs,
}: {
  geometry: Geometry;
  srs: string | null;
}) {
  switch (geometry.type) {
    case "Point":
      return (
        <div>
          Point: (x: {geometry.x}, y: {geometry.y})
        </div>
      );
    case "Line":
      return <LineView {...geometry} />;
    case "LineString":
      return <LineStringView line={geometry} srs={srs} />;
    case "Polygon":
      return <PolygonViewer polygon={geometry} />;
    default:
      return <div>Unknown {geometry.type}</div>;
  }
}

export function LineView({ start, end }: Line) {
  return (
    <div>
      Line:
      <span>
        Start: ({start.x}, {start.y})
      </span>
      <span>
        End: ({end.x}, {end.y})
      </span>
    </div>
  );
}
