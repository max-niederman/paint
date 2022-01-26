import App from "./App.svelte";
import "./styles/global.scss";
import SearchWorker from "./search/worker?worker";

const app = new App({
	target: document.getElementById("app")
});

const searchWorker = new SearchWorker();
console.log(searchWorker);

export default app;
