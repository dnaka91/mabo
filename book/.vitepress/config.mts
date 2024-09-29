import { defineConfig } from "vitepress";
import { generateSidebar } from "vitepress-sidebar";
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

    sidebar: generateSidebar({
      documentRootPath: "src/",
      useTitleFromFileHeading: true,
      useTitleFromFrontmatter: true,
      useFolderTitleFromIndexFile: true,
      sortMenusByFrontmatterOrder: true,
      excludeFilesByFrontmatterFieldName: "exclude",
    }),

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
