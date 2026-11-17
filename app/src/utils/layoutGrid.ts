import type { LayoutMachine } from "@/types/Config";

export interface GridCell {
  x: number;
  y: number;
  key: string;
  isPrimary: boolean;
  machine?: LayoutMachine;
}

const NEIGHBORS = [
  [1, 0],
  [-1, 0],
  [0, 1],
  [0, -1],
];

function cellKey(x: number, y: number) {
  return `${x},${y}`;
}

/** Les machines placées et une rangée libre tout autour */
function gridBounds(machines: LayoutMachine[]) {
  const xs = [0, ...machines.map(machine => machine.x)];
  const ys = [0, ...machines.map(machine => machine.y)];
  return {
    x0: Math.min(...xs) - 1,
    x1: Math.max(...xs) + 1,
    y0: Math.min(...ys) - 1,
    y1: Math.max(...ys) + 1,
  };
}

export function gridColumns(machines: LayoutMachine[]) {
  const bounds = gridBounds(machines);
  return bounds.x1 - bounds.x0 + 1;
}

export function gridCells(machines: LayoutMachine[]): GridCell[] {
  const bounds = gridBounds(machines);
  const cells: GridCell[] = [];
  for (let y = bounds.y0; y <= bounds.y1; y++) {
    for (let x = bounds.x0; x <= bounds.x1; x++) {
      cells.push({
        x,
        y,
        key: cellKey(x, y),
        isPrimary: x === 0 && y === 0,
        machine: machines.find(machine => machine.x === x && machine.y === y),
      });
    }
  }
  return cells;
}

/** Pilotées qu'aucun bord ne relie à la principale : seul un raccourci y mène */
export function isolatedMachines(machines: LayoutMachine[]) {
  const occupied = new Set(machines.map(machine => cellKey(machine.x, machine.y)));
  const reached = new Set([cellKey(0, 0)]);
  const queue = [[0, 0]];
  while (queue.length > 0) {
    const [x, y] = queue.shift() as number[];
    const next = NEIGHBORS.map(([dx, dy]) => [x + dx, y + dy]).filter(
      ([nx, ny]) => occupied.has(cellKey(nx, ny)) && !reached.has(cellKey(nx, ny))
    );
    for (const [nx, ny] of next) {
      reached.add(cellKey(nx, ny));
      queue.push([nx, ny]);
    }
  }
  return machines
    .filter(machine => !reached.has(cellKey(machine.x, machine.y)))
    .map(machine => machine.name);
}
