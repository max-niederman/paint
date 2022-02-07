import svelte from "rollup-plugin-svelte";
import sveltePreprocess from "svelte-preprocess";
import styles from "rollup-plugin-styles";
import typescript from "@rollup/plugin-typescript";
import { nodeResolve as resolve } from "@rollup/plugin-node-resolve";
import commonjs from "@rollup/plugin-commonjs";
import smartAsset from "rollup-plugin-smart-asset";
import staticFiles from "rollup-plugin-static-files";
import { terser } from "rollup-plugin-terser";
import livereload from "rollup-plugin-livereload";
import serve from "rollup-plugin-serve";
import progress from "rollup-plugin-progress";
import del from "rollup-plugin-delete";

const development = process.env.NODE_ENV === "development";

/**
 * @type {import('rollup').RollupOptions}
 */
export default [
    // main bundle
    {
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
                onlyFiles: true, // don't interfere with `dist/search`
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

            // TODO: test incremental builds using
            //       https://github.com/mprt-org/rollup-plugin-incremental

            development && serve({
                port: 4212,
                contentBase: ["dist"],
                historyApiFallback: true,
            }),
            development && livereload("dist"),
        ],
    },
];