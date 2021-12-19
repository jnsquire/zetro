import { terser } from "rollup-plugin-terser";
import typescript from "@rollup/plugin-typescript";

export default {
  input: "src/index.ts",
  output: {
    format: "iife",
    file: "dist/build/bundle.js",
  },
  plugins: [
    typescript(),

    terser({
      // IMPORTANT:
      // We use terser to minify property names. This
      // gives us an even smaller bundle. Zetro will always append an underscore
      // to property names for this readson.
      // {example: ""} becomes {example_: ""}
      mangle: {
        properties: {
          regex: /_$/,
        },
      },
    }),
  ],
};
