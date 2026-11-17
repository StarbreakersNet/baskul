#!/usr/bin/env node
// Conventional Commits (docs/conventions/commits.md) : titre de la PR et messages des commits.
// Usage : node scripts/check-commits.mjs <base> <head> ["titre de la PR"]
import { execFileSync } from "node:child_process";

const TYPES = ["feat", "fix", "perf", "refactor", "style", "test", "docs", "build", "ci", "chore", "revert"];
const SCOPES = ["core", "desktop", "windows", "macos", "linux", "app", "front", "i18n", "deps", "release"];
const MAX_LENGTH = 72;
const HEADER = new RegExp(`^(${TYPES.join("|")})(\\((${SCOPES.join("|")})\\))?!?: \\S`);

export function headerErrors(header) {
  const errors = [];
  if (!HEADER.test(header)) {
    errors.push(`format attendu "type(portée): description" (types : ${TYPES.join(", ")} ; portées : ${SCOPES.join(", ")})`);
  }
  if (header.length > MAX_LENGTH) {
    errors.push(`${header.length} caractères, ${MAX_LENGTH} au plus`);
  }
  if (/\.$/.test(header)) {
    errors.push("pas de point à la fin");
  }
  return errors;
}

function commitHeaders(base, head) {
  const output = execFileSync("git", ["log", "--no-merges", "--format=%h %s", `${base}..${head}`], {
    encoding: "utf8",
  });
  return output
    .split("\n")
    .filter(Boolean)
    .map(line => ({ label: `commit ${line.slice(0, line.indexOf(" "))}`, header: line.slice(line.indexOf(" ") + 1) }));
}

const [base, head, title] = process.argv.slice(2);
if (base && head) {
  const entries = commitHeaders(base, head);
  if (title) {
    entries.unshift({ label: "titre de la PR", header: title });
  }
  let failures = 0;
  for (const { label, header } of entries) {
    const errors = headerErrors(header);
    if (errors.length) {
      failures += 1;
      console.error(`✗ ${label} : "${header}"\n  ${errors.join("\n  ")}`);
    }
  }
  if (failures) {
    console.error(`\n${failures} message(s) hors convention : voir docs/conventions/commits.md`);
    process.exit(1);
  }
  process.stdout.write(`${entries.length} message(s) conformes à Conventional Commits\n`);
}
