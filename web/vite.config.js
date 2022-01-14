import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import { ViteRsw } from "vite-plugin-rsw";

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    svelte(),
    ViteRsw({
      cli: "pnpm",
      profile: "dev", // TODO: detect production
      root: "../packages",
      crates: [{
        name: "glaze-wasm",
      }],
    })
  ],
  server: {
    port: process.env.PORT || 4212,
  }
});
