---
# https://vitepress.dev/reference/default-theme-home-page
layout: home

hero:
  name: "Mabo"
  text: "Strongly Typed Encoding Format"
  tagline: Data format and schema, with a type system as strong as Rust's
  image:
    src: /logo.svg
    alt: Mabo
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
  - icon: 😋
    title: Delicious naming
    details: Named after food, joining projects like <em>Bun</em> and <em>OpenTofu</em>.
---
<style>
:root {
  --vp-home-hero-name-color: transparent;
  --vp-home-hero-name-background: -webkit-linear-gradient(120deg, hwb(45 10% 10%) 30%, hwb(15 10% 10%));

  --vp-home-hero-image-background-image: linear-gradient(-45deg, hwb(45 10% 10%) 30%, hwb(15 10% 10%) 70%);
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
