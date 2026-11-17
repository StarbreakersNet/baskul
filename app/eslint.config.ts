import { globalIgnores } from "eslint/config";
import type { Linter } from "eslint";
import { defineConfigWithVueTs, vueTsConfigs } from "@vue/eslint-config-typescript";
import pluginVue from "eslint-plugin-vue";
import pluginOxlint from "eslint-plugin-oxlint";
import skipFormatting from "@vue/eslint-config-prettier/skip-formatting";

const COMPONENTS_DOC = "voir docs/conventions/composants.md";
const ORGANISATION_DOC = "voir docs/conventions/organisation.md";
const AGENTS_DOC = "voir AGENTS.md";

const NO_TS_EXTENSION = {
  regex: "^@/.*\\.ts$",
  message: `Importer sans l'extension .ts (${COMPONENTS_DOC})`,
};

const NO_RELATIVE_IMPORT = {
  regex: "^\\.{1,2}/",
  message: `Importer par l'alias @/ (${COMPONENTS_DOC})`,
};

const NAIVE_COMPONENT_IMPORTS = {
  group: ["naive-ui"],
  importNamePattern: "^N[A-Z]",
  allowTypeImports: true,
  message: `Utiliser le wrapper h-* de components/c/, à créer s'il n'existe pas (voir docs/conventions/wrappers.md)`,
};

const NAIVE_FEEDBACK_IMPORTS = {
  name: "naive-ui",
  importNames: ["useMessage", "useNotification"],
  message: `Utiliser useAppMessage ou useAppNotification de @/composables/ (${ORGANISATION_DOC})`,
};

const VUE_IMPORTS = {
  name: "vue",
  importNames: ["toRefs", "useAttrs", "PropType"],
  message: `toRefs : un ref d'objet à la place du reactive. useAttrs : v-bind="$attrs" dans le template. PropType : le type en paramètre, defineProps<…>() ou defineModel<…>() (${COMPONENTS_DOC})`,
};

const DAEMON_CLIENT_IMPORT = {
  name: "@/utils/daemonClient",
  message: `Baskul ne s'appelle que depuis les stores : passer par une action de store (${ORGANISATION_DOC})`,
};

function restrictedImports({
  naiveComponents = false,
  naiveFeedback = true,
  daemonClient = true,
} = {}): Linter.RuleEntry {
  return [
    "error",
    {
      paths: [
        VUE_IMPORTS,
        ...(naiveFeedback ? [NAIVE_FEEDBACK_IMPORTS] : []),
        ...(daemonClient ? [DAEMON_CLIENT_IMPORT] : []),
      ],
      patterns: [
        NO_TS_EXTENSION,
        NO_RELATIVE_IMPORT,
        ...(naiveComponents ? [NAIVE_COMPONENT_IMPORTS] : []),
      ],
    },
  ];
}

const SCRIPT_SYNTAX = [
  {
    selector: "ReturnStatement[argument=null]",
    message: `Pas de return vide : écrire la condition en positif, avec la suite dans le if (${COMPONENTS_DOC})`,
  },
  {
    selector: "Literal[value=/[«»]/], TemplateElement[value.raw=/[«»]/]",
    message: `Guillemets droits "…" à la place de « … » (${AGENTS_DOC})`,
  },
  {
    selector:
      "CallExpression[callee.name='useTemplateRef'] > Literal[value!=/Ref$/], VariableDeclarator[init.callee.name='useTemplateRef'] > Identifier.id[name!=/Ref$/]",
    message: `Le nom d'une ref de template finit par Ref : formRef (${AGENTS_DOC})`,
  },
  {
    selector:
      "VariableDeclarator[init.callee.name='useLoader'] > Identifier.id[name!=/^[a-z]+ing([A-Z]\\w*)?$/]",
    message: `Un loader porte le nom de l'action en cours, en -ing : saving, deleting (${COMPONENTS_DOC})`,
  },
  {
    selector: "VariableDeclarator[id.type='ObjectPattern'][init.callee.name='useLoader']",
    message: `Un loader ne se déstructure pas : saving.loading (${COMPONENTS_DOC})`,
  },
  {
    selector:
      "ExpressionStatement > CallExpression[callee.property.name='run'][callee.object.name=/^[a-z]+ing([A-Z]\\w*)?$/]",
    message: `Toujours await devant run() (${COMPONENTS_DOC})`,
  },
  {
    selector:
      "CallExpression[callee.name='defineEmits'] TSPropertySignature > Literal.key[value=/^update:/], CallExpression[callee.name='emit'] > Literal:first-child[value=/^update:/]",
    message: `defineModel à la place d'un événement update:xxx écrit à la main (${COMPONENTS_DOC})`,
  },
  {
    selector:
      "CallExpression[callee.name='defineModel'] > ObjectExpression > Property[key.name='type']",
    message: `Le type en paramètre : defineModel<boolean>("show", { required: true }) (${COMPONENTS_DOC})`,
  },
  {
    selector:
      "VariableDeclarator[init.callee.name='defineStore'] > Identifier.id[name!=/^use[A-Z][A-Za-z]*Store$/]",
    message: `Un store s'appelle useXxxStore (${AGENTS_DOC})`,
  },
  {
    selector: "CallExpression[callee.property.name='then']",
    message: `async / await à la place de .then() (${COMPONENTS_DOC})`,
  },
  {
    selector: "CallExpression[callee.object.name='Object'][callee.property.name='assign']",
    message: `Pas d'Object.assign : un reactive se modifie champ par champ, sinon c'est un ref d'objet (${COMPONENTS_DOC})`,
  },
  // Sélecteurs plutôt que @typescript-eslint/naming-convention, qui ferait passer ESLint en analyse
  // typée : plus lente, elle ignore aussi le /* @vue-ignore */ des wrappers
  {
    selector:
      "TSInterfaceDeclaration > Identifier.id[name!=/^[A-Z][A-Za-z0-9]*$/], TSTypeAliasDeclaration > Identifier.id[name!=/^[A-Z][A-Za-z0-9]*$/]",
    message: `Types et interfaces en PascalCase (${AGENTS_DOC})`,
  },
  {
    selector: "FunctionDeclaration > Identifier.id[name!=/^[a-z][A-Za-z0-9]*$/]",
    message: `Fonctions en camelCase (${AGENTS_DOC})`,
  },
  {
    selector: "FunctionDeclaration > Identifier.id[name=/^on[A-Z]/]",
    message: `Un gestionnaire d'événement s'appelle handleXxx, ou porte le verbe de son action (addLink) : jamais onXxx (${AGENTS_DOC})`,
  },
  {
    selector: "VariableDeclarator > Identifier.id[name!=/^([a-z][A-Za-z0-9]*|[A-Z][A-Z0-9_]*)$/]",
    message: `Variables en camelCase, constantes de module en UPPER_SNAKE_CASE (${AGENTS_DOC})`,
  },
];

const TEMPLATE_SYNTAX = [
  {
    selector: "Identifier[name='$router']",
    message: `Pas de $router dans le template : la navigation se fait dans une fonction du script (${COMPONENTS_DOC})`,
  },
  {
    selector: "Identifier[name='$emit']",
    message: `Émettre avec la constante emit, pas $emit (${COMPONENTS_DOC})`,
  },
  {
    selector: "ConditionalExpression ConditionalExpression",
    message: `Pas de ternaire imbriqué dans le template : un computed ou une fonction (${COMPONENTS_DOC})`,
  },
  {
    selector: "VText[value=/[«»]/], VLiteral[value=/[«»]/]",
    message: `Guillemets droits "…" à la place de « … » (${AGENTS_DOC})`,
  },
];

export default defineConfigWithVueTs(
  {
    name: "app/files-to-lint",
    files: ["**/*.{ts,mts,tsx,vue}"],
  },

  globalIgnores(["**/dist/**", "src-tauri/**"]),

  pluginVue.configs["flat/essential"],
  vueTsConfigs.recommended,
  ...pluginOxlint.configs["flat/recommended"],

  // Règles de docs/conventions/, après oxlint qui désactive les règles ESLint qu'il vérifie déjà
  {
    name: "app/conventions",
    files: ["src/**/*.{ts,vue}"],
    rules: {
      "vue/block-order": ["error", { order: ["script", "template", "style"] }],
      "vue/block-lang": ["error", { script: { lang: "ts" }, style: { lang: "scss" } }],
      "vue/define-macros-order": [
        "error",
        {
          order: ["defineOptions", "defineModel", "defineProps", "defineEmits", "defineSlots"],
          defineExposeLast: true,
        },
      ],
      "vue/component-api-style": ["error", ["script-setup"]],
      "vue/component-name-in-template-casing": ["error", "kebab-case"],
      "vue/no-empty-component-block": "error",
      "vue/require-macro-variable-name": "error",
      "vue/prefer-use-template-ref": "error",
      "vue/no-unused-refs": "error",
      "vue/define-props-declaration": ["error", "type-based"],
      "vue/define-props-destructuring": "error",
      "vue/define-emits-declaration": ["error", "type-literal"],
      "vue/custom-event-name-casing": ["error", "camelCase"],
      "vue/v-on-event-hyphenation": ["error", "always", { autofix: true }],
      "vue/attribute-hyphenation": ["error", "always"],
      "vue/prefer-true-attribute-shorthand": "error",
      "vue/v-on-handler-style": ["error", "inline"],
      "vue/no-restricted-static-attribute": [
        "error",
        {
          key: "ref",
          value: "/^(?!.*Ref$)/",
          message: `Le nom d'une ref de template finit par Ref : formRef (${AGENTS_DOC})`,
        },
      ],
      "vue/no-restricted-syntax": ["error", ...TEMPLATE_SYNTAX],
      "no-restricted-syntax": ["error", ...SCRIPT_SYNTAX],
      "no-restricted-imports": restrictedImports(),
      "no-warning-comments": ["error", { terms: ["«", "»"], location: "anywhere" }],
      "no-console": ["error", { allow: ["warn", "error"] }],
      "no-continue": "error",
      "no-fallthrough": "error",
      "max-depth": ["error", 3],
      "func-style": ["error", "declaration"],
      "@typescript-eslint/consistent-type-definitions": ["error", "interface"],
    },
  },

  // App.vue garde les providers Naive UI (n-config-provider…)
  {
    name: "app/wrappers-only",
    files: ["src/**/*.vue"],
    ignores: ["src/components/c/**", "src/App.vue"],
    rules: {
      "vue/no-undef-components": [
        "error",
        // Composants globaux qui ne viennent pas de Naive UI
        { ignorePatterns: ["^router-(view|link)$", "^i18n-t$"] },
      ],
      "no-restricted-imports": restrictedImports({ naiveComponents: true }),
    },
  },

  // AppLayout est sous les providers de messages et de notifications : il transmet leur API aux composables
  {
    name: "app/feedback-providers",
    files: ["src/AppLayout.vue"],
    rules: {
      "no-restricted-imports": restrictedImports({ naiveComponents: true, naiveFeedback: false }),
    },
  },

  {
    name: "app/stores",
    files: ["src/stores/**/*.ts"],
    rules: {
      "no-restricted-imports": restrictedImports({ daemonClient: false }),
    },
  },

  {
    name: "app/models",
    files: ["src/models/**/*.ts"],
    rules: {
      "no-restricted-syntax": [
        "error",
        ...SCRIPT_SYNTAX,
        {
          selector:
            "ExportNamedDeclaration > VariableDeclaration > VariableDeclarator > Identifier.id[name!=/^[A-Z][A-Z0-9_]*$/]",
          message: `Constantes de models/ en UPPER_SNAKE_CASE : MENU_ITEMS (${ORGANISATION_DOC})`,
        },
        {
          selector: "Program > FunctionDeclaration, ExportNamedDeclaration > FunctionDeclaration",
          message: `Pas de fonction dans models/ : elle va dans utils/ (${ORGANISATION_DOC})`,
        },
      ],
    },
  },

  skipFormatting
);
