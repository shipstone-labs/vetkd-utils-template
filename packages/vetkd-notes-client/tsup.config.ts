import { defineConfig } from "tsup";

export default defineConfig({
  entry: ["src/index.ts"], // Add your TypeScript entry points
  external: ["src/vetkd_notes_client.wasm", "src/vetkd_notes_client.wasm.d.ts"],
  outDir: "dist", // Directory for compiled output
  format: ["esm", "cjs"], // Output formats (CommonJS and ESM)
  sourcemap: true, // Enable sourcemaps if needed
  dts: true, // Generate .d.ts declaration files
  clean: true, // Clean the output directory before building
});
