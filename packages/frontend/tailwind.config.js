module.exports = {
  content: [
    "./public/index.html",
    "./src/**/*.{html,js,svelte,ts}",
    "./node_modules/daisyui/dist/**/*.js",
    "./node_modules/daisyui/**/*.js",
  ],
  theme: {
    extend: {},
  },
  plugins: [require("daisyui")],
};
