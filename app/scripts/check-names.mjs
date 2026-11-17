#!/usr/bin/env node
// Noms de fichiers de src/ (docs/conventions/organisation.md et AGENTS.md), qu'ESLint ne vérifie pas.
// Lancé par yarn lint et par la CI. Aucune dépendance.

import { existsSync, readdirSync, readFileSync } from "node:fs";
import { basename, join, relative } from "node:path";
import { fileURLToPath } from "node:url";

const SRC = fileURLToPath(new URL("../src/", import.meta.url));

const RULES = [
  {
    dir: "views",
    ext: ".vue",
    pattern: /^[A-Z][A-Za-z0-9]*View$/,
    expected: "PascalCase avec le suffixe View : AccountView.vue",
  },
  {
    dir: "components",
    ext: ".vue",
    exclude: "components/c",
    pattern: /^[A-Z][a-z0-9]+(?:[A-Z][a-z0-9]*)+$/,
    expected: "PascalCase, au moins deux mots : UserMenu.vue",
  },
  {
    dir: "components/c",
    ext: ".vue",
    pattern: /^h[A-Z][A-Za-z0-9]*$/,
    expected: "hXxx : hButton.vue",
  },
  {
    dir: "composables",
    ext: ".ts",
    pattern: /^use[A-Z][A-Za-z0-9]*$/,
    expected: "useXxx : useLoader.ts",
  },
  {
    dir: "stores",
    ext: ".ts",
    pattern: /^[a-z][A-Za-z0-9]*$/,
    expected: "camelCase, au nom de l'id du store : auth.ts",
    check: checkStore,
  },
  {
    dir: "utils",
    ext: ".ts",
    pattern: /^(?!utils$|helpers$)[a-z][A-Za-z0-9]*$/,
    expected: "camelCase, au nom du sujet (jamais utils.ts ni helpers.ts) : userTag.ts",
  },
  {
    dir: "models",
    ext: ".ts",
    pattern: /^[A-Z][A-Za-z0-9]*$/,
    expected: "PascalCase, au nom du type : MenuItem.ts",
  },
  {
    dir: "types",
    ext: ".ts",
    exclude: "types/components/c",
    pattern: /^[A-Z][A-Za-z0-9]*$/,
    expected: "PascalCase : User.ts",
  },
  {
    dir: "types/components/c",
    ext: ".ts",
    pattern: /^h[A-Z][A-Za-z0-9]*$/,
    expected: "au nom du wrapper : hForm.ts",
  },
];

function listFiles(dir) {
  if (!existsSync(dir)) {
    return [];
  }
  return readdirSync(dir, { withFileTypes: true }).flatMap(entry =>
    entry.isDirectory() ? listFiles(join(dir, entry.name)) : [join(dir, entry.name)]
  );
}

function pascalCase(name) {
  return name.charAt(0).toUpperCase() + name.slice(1);
}

// Le store auth.ts déclare defineStore("auth", …) et l'exporte sous le nom useAuthStore
function checkStore(file, name) {
  const source = readFileSync(file, "utf8");
  const expected = `export const use${pascalCase(name)}Store = defineStore("${name}"`;
  return source.includes(expected) ? [] : [`doit contenir ${expected}, …)`];
}

function checkRule(rule) {
  const dir = join(SRC, rule.dir);
  const files = listFiles(dir).filter(
    file =>
      file.endsWith(rule.ext) && !(rule.exclude && relative(SRC, file).startsWith(rule.exclude))
  );

  return files.flatMap(file => {
    const name = basename(file, rule.ext);
    const errors = rule.pattern.test(name) ? (rule.check?.(file, name) ?? []) : [rule.expected];
    return errors.map(error => `src/${relative(SRC, file)} : ${error}`);
  });
}

const errors = RULES.flatMap(checkRule);

if (errors.length > 0) {
  console.error(`Noms de fichiers (docs/conventions/organisation.md) :\n${errors.join("\n")}`);
  process.exitCode = 1;
}
