import { confirm, message } from "@tauri-apps/api/dialog";
import { useState } from "react";
import { getCsv, Point } from "./bindings";
import { openFile } from "./files";

type Record = { point: Point; file: string };

export const ThiessenPolygons = () => {
  const [records, setRecords] = useState<Record[]>([]);
  const addRecord = (record: Record) => {
    setRecords([...records, record]);
  };
  const clearRecords = () => {
    confirm(
      "Are you sure you want to clear the currently added locations?"
    ).then((x) => {
      if (x === true) {
        setRecords([]);
      }
    });
  };
  return (
    <div>
      <h2>Thiessen Polygons method for calculating rainfall</h2>
      <RecordTable records={records} />
      <RecordAdder add={addRecord} />
      <button disabled={records.length === 0} onClick={clearRecords}>
        Clear
      </button>
      <button>Calculate</button>
    </div>
  );
};

const RecordTable = ({ records }: { records: Record[] }) => {
  return (
    <table>
      <thead>
        <tr>
          <th>File</th>
          <th>Latitude</th>
          <th>Longitude</th>
        </tr>
      </thead>
      <tbody>
        {records.map((record) => (
          <tr>
            <td>{record.file}</td>
            <td>{record.point.y}</td>
            <td>{record.point.x}</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
};

export const RecordAdder = ({ add }: { add: (_: Record) => void }) => {
  const [file, setFile] = useState<string | null>(null);
  const [csv, setCsv] = useState<string[][]>([]);
  const [selectingCell, setSelectingCell] = useState(false);
  const [lat, setLat] = useState(0);
  const [long, setLong] = useState(0);
  const [_cell, setCell] = useState({ line: 0, column: 0 });
  const loadFile = async () => {
    const file = await openFile("Csv to read data from");
    if (file !== null) {
      setFile(file);
      await getCsv(file)
        .then((csv) => {
          setCsv(csv);
          setSelectingCell(true);
        })
        .catch((e) => message(e as string));
    }
  };

  const selectCellHandler = (cell: { line: number; column: number }) => {
    setCell(cell);
    setSelectingCell(false);
  };
  const addHandler = () => {
    if (file !== null) {
      add({ point: { x: long, y: lat }, file });
      setFile(null);
      setLat(0);
      setLong(0);
    }
  };
  return selectingCell ? (
    <CsvViewer data={csv} selectCell={selectCellHandler} />
  ) : (
    <div>
      <button onClick={loadFile}>
        {file === null ? "File to load?" : file}
      </button>
      <label>
        Latitude:
        <input
          type=" number"
          value={lat}
          onChange={(e) => setLat(Number(e.target.value))}
        />
      </label>
      <label>
        Longitude:
        <input
          type=" number"
          value={long}
          onChange={(e) => setLong(Number(e.target.value))}
        />
      </label>
      <button onClick={addHandler}>Add</button>
    </div>
  );
};

type CsvViewerProps = {
  data: string[][];
  selectCell: (position: { line: number; column: number }) => void;
};

const CsvViewer = ({ data, selectCell }: CsvViewerProps) => {
  return (
    <table>
      <thead>
        <tr></tr>
      </thead>
      <tbody>
        {data.map((row, line) => (
          <tr>
            {row.map((val, column) => (
              <td>
                <span onClick={() => selectCell({ line, column })}>{val}</span>
              </td>
            ))}
          </tr>
        ))}
      </tbody>
    </table>
  );
};
