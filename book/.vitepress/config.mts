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
  head: [
    ["link", { rel: "icon", type: "image/svg+xml", href: "/mabo/logo.svg" }],
    ["meta", { name: "color-scheme", content: "dark light" }],
    ["meta", { name: "theme-color", content: "#d95a00" }],
    ["meta", { name: "og:type", content: "website" }],
    ["meta", { name: "og:locale", content: "en" }],
    ["meta", { name: "og:site_name", content: "Mabo" }],
  ],
  transformPageData(pageData, ctx) {
    pageData.frontmatter.head ??= [];
    pageData.frontmatter.head.push([
      "meta",
      {
        name: "og:title",
        content:
          pageData.frontmatter.layout === "home"
            ? ctx.siteConfig.site.title
            : `${pageData.title} | ${ctx.siteConfig.site.title}`,
      },
    ]);
  },
  themeConfig: {
    // https://vitepress.dev/reference/default-theme-config
    logo: { src: "/logo.svg", width: 24, height: 24 },
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
              { text: "mabo init", link: "/reference/cli/init" },
              { text: "mabo check", link: "/reference/cli/check" },
              { text: "mabo fmt", link: "/reference/cli/fmt" },
              { text: "mabo doc", link: "/reference/cli/doc" },
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
              // { text: "Statics", link: "/reference/schema/statics" },
              { text: "Aliases", link: "/reference/schema/aliases" },
              { text: "Modules", link: "/reference/schema/modules" },
              { text: "Imports", link: "/reference/schema/imports" },
              // { text: "References", link: "/reference/schema/references" },
              { text: "Attributes", link: "/reference/schema/attributes" },
            ],
          },
          {
            text: "Project Files",
            link: "/reference/project/",
            items: [{ text: "Packages", link: "/reference/project/packages" }],
          },
          {
            text: "Wire Format",
            link: "/reference/wire-format",
          },
          // {
          //   text: "Compiler",
          //   link: "/reference/compiler",
          // },
          {
            text: "Generators",
            link: "/reference/generators/",
            items: [
              { text: "Doc", link: "/reference/generators/doc" },
              { text: "Rust", link: "/reference/generators/rust" },
              { text: "Go", link: "/reference/generators/go" },
              { text: "Kotlin", link: "/reference/generators/kotlin" },
              { text: "TypeScript", link: "/reference/generators/typescript" },
              { text: "Python", link: "/reference/generators/python" },
            ],
          },
        ],
      },
      {
        text: "Miscellaneous",
        items: [
          { text: "Team", link: "/misc/team" },
          { text: "License", link: "/misc/license" },
          { text: "Changelog", link: "/misc/changelog" },
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

    externalLinkIcon: true,
  },
});
