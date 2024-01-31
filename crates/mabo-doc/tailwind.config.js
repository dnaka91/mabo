const defaultTheme = require("tailwindcss/defaultTheme")
const colors = require("tailwindcss/colors")

/** @type {import('tailwindcss').Config} */
export default {
  content: {
    relative: true,
    files: ["./templates/**/*.html"],
  },
  theme: {
    extend: {
      colors: {
        main: colors.neutral,
      },
      fontFamily: {
        sans: ['"Noto Sans"', ...defaultTheme.fontFamily.sans],
        serif: ['"Noto Serif"', ...defaultTheme.fontFamily.serif],
        mono: ['"Fira Code"', ...defaultTheme.fontFamily.mono],
      },
    },
  },
  plugins: [
    require("@tailwindcss/typography")({
      className: "markdown",
    }),
  ],
}
