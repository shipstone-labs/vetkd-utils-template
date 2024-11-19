module.exports = {
  content: ["./public/index.html", "./src/**/*.{html,js,svelte,ts}"],
  theme: {
    extend: {},
  },
  plugins: [require("daisyui")],
  daisyui: {
    // Try disabling base, styled, or utilities if necessary
    base: true, // Disable DaisyUI base styles
    styled: false, // Disable DaisyUI styled components
    utilities: true, // Disable DaisyUI utilities
  },
};
