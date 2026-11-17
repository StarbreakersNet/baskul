import { describe, expect, it } from "vitest";
import { gridCells, gridColumns, isolatedMachines } from "@/utils/layoutGrid";

describe("grille des machines", () => {
  it("entoure les machines placées d'une rangée libre", () => {
    const machines = [{ name: "Atelier", x: 1, y: 0 }];
    expect(gridColumns(machines)).toBe(4);
    const cells = gridCells(machines);
    expect(cells).toHaveLength(12);
    expect(cells.find(cell => cell.isPrimary)?.key).toBe("0,0");
    expect(cells.find(cell => cell.machine)?.machine?.name).toBe("Atelier");
  });

  it("repère les pilotées qu'aucun bord ne relie à la principale", () => {
    const machines = [
      { name: "Atelier", x: 1, y: 0 },
      { name: "Salon", x: 2, y: 0 },
      { name: "Portable", x: 0, y: -2 },
    ];
    expect(isolatedMachines(machines)).toEqual(["Portable"]);
  });

  it("ne relie pas deux machines en diagonale", () => {
    expect(isolatedMachines([{ name: "Coin", x: 1, y: 1 }])).toEqual(["Coin"]);
  });
});
