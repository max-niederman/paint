import App from "./App.svelte";
import "./styles/global.scss";
import init, { SearchManager } from "glaze-wasm";

const app = new App({
	target: document.getElementById("app")
});

await init();

const searchManager = new SearchManager();

export default app;
