import { describe, expect, it } from "vitest";
import { filterLogLines, LOG_LANGUAGE, logHighlighter, parseLogLines } from "@/utils/logLines";

const LOG = [
  "2026-11-16 23:05:43.806 INFO  [baskul_desktop] Baskul 0.8.0",
  "2026-11-16 23:06:10.120 WARN  [baskul_desktop::clipboard] image trop grande",
  "2026-11-16 23:06:12.500 ERROR [baskul_desktop::net] connexion perdue",
  "  suite du message sur une autre ligne",
  "2026-11-16 23:06:13.000 TRACE [baskul_core::switch] bord droit",
  "",
].join("\n");

describe("journal", () => {
  it("lit le niveau de chaque ligne", () => {
    expect(parseLogLines(LOG).map(line => line.level)).toEqual([
      "info",
      "warn",
      "error",
      "error",
      "debug",
    ]);
  });

  it("filtre par niveau et par texte, sans tenir compte de la casse", () => {
    const lines = parseLogLines(LOG);
    expect(filterLogLines(lines, ["error"], "")).toHaveLength(2);
    expect(filterLogLines(lines, ["info", "warn", "error", "debug"], "IMAGE")).toHaveLength(1);
    expect(filterLogLines(lines, ["info"], "image")).toHaveLength(0);
  });

  it("colore les lignes et échappe le HTML", () => {
    expect(logHighlighter.getLanguage(LOG_LANGUAGE)).toBeDefined();
    expect(logHighlighter.getLanguage("rust")).toBeUndefined();
    const html = logHighlighter.highlight(
      "2026-11-16 23:06:12.500 ERROR [net] <b>perdue</b>\nautre"
    ).value;
    expect(html).toContain('<span class="hljs-deletion">ERROR</span>');
    expect(html).toContain('<span class="hljs-title">[net]</span>');
    expect(html).toContain("&lt;b&gt;perdue&lt;/b&gt;");
    expect(html.split("\n")[1]).toBe("autre");
  });
});
