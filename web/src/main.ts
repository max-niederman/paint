import App from "./App.svelte";
import "./styles/global.scss";
import initWasm, { SearchManager } from "glaze-wasm";
import type { View } from "glaze-wasm";

const app = new App({
	target: document.getElementById("app")
});

await initWasm();

let searchManager = await new SearchManager();
const view: View = { 
		truth: { base_url: "https://lms.pps.net" }, 
		viewer: { User: 89090000000116506 }
	};
console.log(searchManager.update(view))
console.log(searchManager.query(view, { text: "" }));

export default app;
