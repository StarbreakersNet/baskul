export type LogLevel = "error" | "warn" | "info" | "debug";

export interface LogLine {
  level: LogLevel;
  text: string;
}

export const LOG_LEVELS: LogLevel[] = ["error", "warn", "info", "debug"];

export const LOG_LANGUAGE = "baskul-log";

// Lignes écrites par src/logging.rs : "2026-01-01 12:00:00.000 INFO  [cible] message"
const LINE_PATTERN = /^(\S+ \S+) (ERROR|WARN|INFO|DEBUG|TRACE)\s+(\[[^\]]*\])?(.*)$/;

function levelOf(name: string): LogLevel {
  if (name === "TRACE") {
    return "debug";
  }
  return name.toLowerCase() as LogLevel;
}

/** Une ligne sans en-tête (suite d'un message sur plusieurs lignes) garde le niveau de la précédente */
export function parseLogLines(text: string): LogLine[] {
  let level: LogLevel = "info";
  return text
    .split("\n")
    .filter(line => line.trim() !== "")
    .map(line => {
      const match = LINE_PATTERN.exec(line);
      if (match) {
        level = levelOf(match[2]);
      }
      return { level, text: line };
    });
}

export function filterLogLines(lines: LogLine[], levels: LogLevel[], search: string) {
  const needle = search.trim().toLowerCase();
  return lines.filter(
    line => levels.includes(line.level) && (!needle || line.text.toLowerCase().includes(needle))
  );
}

function escapeHtml(text: string) {
  return text.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}

const LEVEL_CLASSES: Record<string, string> = {
  ERROR: "hljs-deletion",
  WARN: "hljs-number",
  INFO: "hljs-string",
  DEBUG: "hljs-comment",
  TRACE: "hljs-comment",
};

function highlightLine(line: string) {
  const match = LINE_PATTERN.exec(line);
  if (!match) {
    return escapeHtml(line);
  }
  const [, time, level, target = "", rest] = match;
  const padding = line.slice(time.length + 1 + level.length).match(/^\s*/)?.[0] ?? " ";
  return [
    `<span class="hljs-comment">${escapeHtml(time)}</span> `,
    `<span class="${LEVEL_CLASSES[level]}">${level}</span>${padding}`,
    target ? `<span class="hljs-title">${escapeHtml(target)}</span>` : "",
    escapeHtml(rest),
  ].join("");
}

/** Coloration des lignes du journal, au format attendu par NLog à la place de highlight.js */
export const logHighlighter = {
  getLanguage: (name: string) => (name === LOG_LANGUAGE ? {} : undefined),
  highlight: (code: string) => ({ value: code.split("\n").map(highlightLine).join("\n") }),
};
