import tsParser from "@typescript-eslint/parser";
import prettier from "eslint-config-prettier";
import svelte from "eslint-plugin-svelte";

export default [
  {
    ignores: [
      "node_modules/**",
      "build/**",
      ".svelte-kit/**",
      "package/**",
      "src-tauri/target/**",
      "src-tauri/gen/**",
    ],
  },
  ...svelte.configs["flat/recommended"],
  {
    files: ["**/*.ts"],
    languageOptions: {
      parser: tsParser,
      parserOptions: {
        ecmaVersion: "latest",
        sourceType: "module",
      },
    },
  },
  {
    files: ["**/*.svelte"],
    languageOptions: {
      parserOptions: {
        parser: tsParser,
      },
    },
  },
  {
    linterOptions: {
      reportUnusedDisableDirectives: false,
    },
    rules: prettier.rules,
  },
];
