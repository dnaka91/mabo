---
# https://vitepress.dev/reference/default-theme-home-page
layout: home

hero:
  name: "Stef"
  text: "Strongly Typed Encoding Format"
  tagline: Data format and schema, with a type system as strong as Rust's
  image:
    src: /logo.svg
    alt: Stef
  actions:
    - theme: brand
      text: User Guide
      link: /guide/installation
    - theme: alt
      text: Reference
      link: /reference/schema/

features:
  - icon: 🦀
    title: Type safe
    details: Strongly typed schemas with a type system like Rust's.
  - icon: 🧰
    title: Rich tooling
    details: Ships with many tools like formatter, linter and documentation generator and more.
---
<style>
:root {
  --vp-home-hero-name-color: transparent;
  --vp-home-hero-name-background: -webkit-linear-gradient(120deg, #3f3f74 30%, #5b6ee1);

  --vp-home-hero-image-background-image: linear-gradient(-45deg, #3f3f74 50%, #5b6ee1 50%);
  --vp-home-hero-image-filter: blur(44px);
}

@media (min-width: 640px) {
  :root {
    --vp-home-hero-image-filter: blur(56px);
  }
}

@media (min-width: 960px) {
  :root {
    --vp-home-hero-image-filter: blur(68px);
  }
}
</style>
