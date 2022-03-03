import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

// https://vitejs.dev/config/
export default defineConfig({
  server: {
    port: 4210,
  },
	plugins: [
    svelte(),
    // TODO: generate and include pollen-css with a plugin
  ],
});
