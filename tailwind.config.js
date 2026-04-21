/** @type {import('tailwindcss').Config} */
module.exports = {
  content: { 
    files: ["*.html", "./crates/**/*.rs"],
  },
  theme: {
      extend: {},
  },
  plugins: [
    require('daisyui'),
    require('@tailwindcss/typography')
  ],
  daisyui: {
    themes: [
      {
        emerald: {
          ...require("daisyui/src/theming/themes")["emerald"],
          "nav, .menu": {
            "background-color": "theme(colors.blue.100)",
          },
          "--rounded-btn": "0.25rem",
          "--tab-radius": "0.25rem",
        },
        dim: {
          ...require("daisyui/src/theming/themes")["dim"],
          "nav, .menu": {
            "background-color": "theme(colors.sky.900)",
          },
          "--rounded-btn": "0.25rem",
          "--tab-radius": "0.25rem",
        },
      },
    ],
  },
  darkMode: ['class', '[data-theme="dim"]']
}
