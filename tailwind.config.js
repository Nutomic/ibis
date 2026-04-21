/** @type {import('tailwindcss').Config} */
module.exports = {
  content: {
    relative: true,
    files: ["*.html", "./src/**/*.rs"],
    transform: {
      rs: (content) => content.replace(/(?:^|\s)class:/g, ' '),
    },
  },
  theme: {
    extend: {},
  },
  plugins: [],
}
