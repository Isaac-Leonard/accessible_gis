import { LineString } from "./bindings";

export const PointsTableView = ({ line }: { line: LineString }) => {
  return (
    <table>
      <thead>
        <tr>
          <th>X</th>
          <th>Y</th>
        </tr>
      </thead>
      <tbody>
        {line.points.map((p, i) => (
          <tr key={i.toString() + p.x + p.y}>
            <td>{p.x}</td>
            <td>{p.y}</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
};
