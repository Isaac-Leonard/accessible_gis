import { useEffect, useState } from "react";
import {
  LayerDescriptor,
  pointInCountry,
  nearestTown,
  getPointOfMaxValue,
  getPointOfMinValue,
  getValueAtPoint,
  Point,
} from "./bindings";
import { message } from "@tauri-apps/api/dialog";

export const RasterNavigator = ({
  layer,
}: {
  layer: Extract<LayerDescriptor, { type: "Raster" }>;
}) => {
  let { width, length } = layer;
  const [showCoords, setShowCoords] = useState(true);
  const [{ x, y }, setCoords] = useState({ x: 0, y: 0 });
  const [radius, setRadius] = useState(1);
  const [points, setPoints] = useState<Point[]>([]);
  const [info, setInfo] = useState("");

  useEffect(() => {
    if (x < width && x >= 0 && y < length && y >= 0) {
      getValueAtPoint({ x, y }, layer)
        .then((val) =>
          setInfo(
            showCoords
              ? `${
                  typeof val === "number" ? val.toPrecision(4) : val
                } at ${x}, ${length - y}`
              : typeof val === "number"
              ? val.toPrecision(4)
              : val
          )
        )
        .catch((e) => message(e as string));
    }
  }, [width, length, radius, x, y]);
  return (
    <div
      onKeyDown={(e) => {
        if (e.key === "M") {
          e.preventDefault();
          getPointOfMaxValue(layer).then((p) => setCoords(p));
        }
        if (e.key === "m") {
          e.preventDefault();
          getPointOfMinValue(layer).then((p) => setCoords(p));
        }
        if (e.key === "c") {
          e.preventDefault();
          pointInCountry(layer, { x, y })
            .then((x) => {
              setInfo(
                `In ${x?.name}, ${x?.distance ?? 0 / 1000}km from boarder`
              );
            })
            .catch((e) => message(e));
        }
        if (e.key === "t") {
          e.preventDefault();
          nearestTown(layer, { x, y })
            .then((x) => {
              setInfo(`${x?.distance ?? 0 / 1000}km from ${x?.name}`);
            })
            .catch((e) => message(e));
        }
        if (e.key === "p") {
          e.preventDefault();
          setPoints([...points, { x, y }]);
        }
      }}
    >
      <input
        type="number"
        value={radius}
        onChange={(e) => setRadius(Number(e.target.value))}
      />
      <label>
        Show coords?{" "}
        <input
          type="checkbox"
          defaultChecked={true}
          onChange={(e) => setShowCoords(e.target.checked)}
        />
      </label>
      <div onKeyDown={coordinateArrowHandler(x, y, radius, setCoords)}>
        <CoordinateButtons x={x} y={y} radius={radius} setCoords={setCoords} />
        <p role="status">{info}</p>
      </div>
    </div>
  );
};

type CoordProps = {
  x: number;
  y: number;
  radius: number;
  setCoords: (_: { x: number; y: number }) => void;
};

function CoordinateButtons({ x, y, radius, setCoords }: CoordProps) {
  return (
    <div>
      <button onClick={() => setCoords({ x, y: y - radius })}>Up</button>
      <button onClick={() => setCoords({ x, y: y + radius })}>Down</button>
      <button onClick={() => setCoords({ x: x - radius, y })}>Left</button>
      <button onClick={() => setCoords({ x: x + radius, y })}>Right</button>
    </div>
  );
}

function coordinateArrowHandler(
  x: number,
  y: number,
  radius: number,
  setCoords: (_: { x: number; y: number }) => void
) {
  return (e: React.KeyboardEvent) => {
    if (e.key.startsWith("Arrow")) {
      e.preventDefault();
      switch (e.key) {
        case "ArrowUp":
          setCoords({ x, y: y + radius });
          break;
        case "ArrowDown":
          setCoords({ x, y: y - radius });
          break;
        case "ArrowLeft":
          setCoords({ x: x - radius, y });
          break;
        case "ArrowRight":
          setCoords({ x: x + radius, y });
          break;
      }
    }
  };
}
