import { describe, expect, it } from "vitest";
import { hostWithoutDefaultPort, shellCommand } from "@/utils/address";

describe("adresses", () => {
  it("retire le port par défaut et les crochets IPv6", () => {
    expect(hostWithoutDefaultPort("192.168.1.20:24870", 24870)).toBe("192.168.1.20");
    expect(hostWithoutDefaultPort("[fe80::1]:24870", 24870)).toBe("fe80::1");
    expect(hostWithoutDefaultPort("192.168.1.20:9000", 24870)).toBe("192.168.1.20:9000");
  });

  it("met le chemin de l'exécutable entre guillemets si besoin", () => {
    expect(shellCommand("/usr/bin/baskul", "status", "linux")).toBe("/usr/bin/baskul status");
    expect(shellCommand("C:\\Program Files\\Baskul\\baskul.exe", "status", "windows")).toBe(
      '"C:\\Program Files\\Baskul\\baskul.exe" status'
    );
    expect(shellCommand("/Applications/Mon Baskul/baskul", "switch here", "macos")).toBe(
      "'/Applications/Mon Baskul/baskul' switch here"
    );
  });
});
