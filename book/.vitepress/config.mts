import { defineConfig } from "vitepress";
import maboGrammar from "../../vscode-extension/syntaxes/mabo.tmLanguage.json";

// https://vitepress.dev/reference/site-config
export default defineConfig({
  title: "Mabo",
  description: "Strongly Typed Encoding Format",
  appearance: "dark",
  lastUpdated: true,
  srcDir: "src",
  base: "/mabo/",
  markdown: {
    theme: {
      dark: "one-dark-pro",
      light: "min-light",
    },
    languages: [
      {
        // biome-ignore lint: the grammar is wrongly reported as incompatible
        ...(maboGrammar as any),
        name: "mabo",
      },
    ],
    lineNumbers: true,
    image: {
      lazyLoading: true,
    },
  },
  vite: {
    resolve: {
      preserveSymlinks: true,
    },
  },
  head: [["link", { rel: "icon", type: "image/svg+xml", href: "/mabo/logo.svg" }]],
  themeConfig: {
    // https://vitepress.dev/reference/default-theme-config
    logo: "/logo.svg",
    editLink: {
      pattern: "https://github.com/dnaka91/mabo/edit/main/book/src/:path",
      text: "Edit this page on GitHub",
    },
    nav: [
      { text: "Guide", link: "/guide/installation", activeMatch: "/guide/" },
      { text: "Reference", link: "/reference/schema/", activeMatch: "/reference/" },
    ],

    sidebar: [
      { text: "Introduction", link: "/introduction" },
      { text: "Ideas", link: "/ideas" },
      {
        text: "User Guide",
        items: [
          { text: "Installation", link: "/guide/installation" },
          { text: "Creating schemas", link: "/guide/creating" },
          { text: "Generating code", link: "/guide/generating" },
          { text: "Examples", link: "/guide/examples" },
        ],
      },
      {
        text: "Reference",
        items: [
          {
            text: "Command Line Interface",
            link: "/reference/cli/",
            items: [
              { text: "mabo lint", link: "/reference/cli/lint" },
              { text: "mabo check", link: "/reference/cli/check" },
              { text: "mabo format", link: "/reference/cli/format" },
            ],
          },
          {
            text: "Schema",
            link: "/reference/schema/",
            items: [
              { text: "Structs", link: "/reference/schema/structs" },
              { text: "Enums", link: "/reference/schema/enums" },
              { text: "Arrays", link: "/reference/schema/arrays" },
              { text: "Tuples", link: "/reference/schema/tuples" },
              { text: "Constants", link: "/reference/schema/constants" },
              { text: "Statics", link: "/reference/schema/statics" },
              { text: "Type Aliases", link: "/reference/schema/type-aliases" },
              { text: "Modules", link: "/reference/schema/modules" },
              { text: "Imports", link: "/reference/schema/imports" },
              { text: "References", link: "/reference/schema/references" },
              { text: "Attributes", link: "/reference/schema/attributes" },
            ],
          },
          {
            text: "Wire Format",
            link: "/reference/wire-format",
          },
          {
            text: "Compiler",
            link: "/reference/compiler",
          },
          {
            text: "Generators",
            link: "/reference/generators/",
            items: [
              { text: "Rust", link: "/reference/generators/rust" },
              { text: "Go", link: "/reference/generators/go" },
            ],
          },
        ],
      },
      {
        text: "Miscellaneous",
        items: [
          { text: "Team", link: "/misc/team" },
          { text: "License", link: "/misc/license" },
        ],
      },
    ],

    outline: "deep",

    socialLinks: [{ icon: "github", link: "https://github.com/dnaka91/mabo" }],

    footer: {
      message: "Released under the MIT License.",
      copyright: "Copyright © 2023-present Dominik Nakamura",
    },

    search: {
      provider: "local",
    },
  },
});
