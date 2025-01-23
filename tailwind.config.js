/** @type {import('tailwindcss').Config} */
module.exports = {
  content: { 
    files: ["*.html", "./src/**/*.rs"],
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
            "background-color": "theme(colors.sky.50)",
          },
        },
        dim: {
          ...require("daisyui/src/theming/themes")["dim"],
          "nav, .menu": {
            "background-color": "theme(colors.sky.900)",
          },
        },
      },
    ],
  },
  darkMode: ['class', '[data-theme="dim"]']
}
