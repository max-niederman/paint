import svelte from "rollup-plugin-svelte";
import sveltePreprocess from "svelte-preprocess";
import styles from "rollup-plugin-styles";
import typescript from "@rollup/plugin-typescript";
import { wasm } from "@rollup/plugin-wasm";
import { nodeResolve as resolve } from "@rollup/plugin-node-resolve";
import commonjs from "@rollup/plugin-commonjs";
import staticFiles from "rollup-plugin-static-files";
import { terser } from "rollup-plugin-terser";
import livereload from "rollup-plugin-livereload";
import serve from "rollup-plugin-serve";
import progress from "rollup-plugin-progress";
import del from "rollup-plugin-delete";

const development = process.env.NODE_ENV === "development";

export default {
    input: "src/main.ts",
    output: {
        dir: "dist",
        format: "iife",
        entryFileNames: "[name].[hash].js",
        assetFileNames: "[name].[hash][extname]",
        sourcemap: development,
    },
    plugins: [
        del({
            targets: ["dist/*"],
        }),

        progress(),

        svelte({
            preprocess: sveltePreprocess({
                sourceMap: development,
            }),
            compilerOptions: {
                dev: development,
            },
        }),
        typescript(),
        wasm(),
        styles({
            mode: [
                "extract",
                "styles.css",
            ],
            minimize: !development,
        }),

        resolve({
            browser: true,
            dedupe: ["svelte"],
        }),
        commonjs(),

        staticFiles({
            include: ["./public"],
        }),

        !development && terser(),

        development && serve({
            port: 4212,
            contentBase: ["dist"],
            historyApiFallback: true,
        }),
        development && livereload("dist"),
    ],
};