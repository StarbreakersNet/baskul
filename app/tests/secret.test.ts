import { describe, expect, it } from "vitest";
import { newSecret } from "@/utils/secret";

describe("secret partagé", () => {
  it("a le format du secret créé par Baskul", () => {
    expect(newSecret()).toMatch(/^([a-hj-km-np-z2-9]{4}-){5}[a-hj-km-np-z2-9]{4}$/);
  });

  it("change à chaque fois", () => {
    expect(newSecret()).not.toBe(newSecret());
  });
});
