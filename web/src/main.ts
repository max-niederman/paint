import App from "./App.svelte";
import "./styles/global.scss";
import SearchWorkerURL from "./search/worker.ts?url";
import type * as Search from "./search/worker";

const app = new App({
	target: document.getElementById("app")
});

const searchWorker = new Worker(SearchWorkerURL, { type: "module" });

searchWorker.onmessage = (msg) => {
	const resp: Search.Response = msg.data;

	console.log(`received response from search worker: ${resp}`);

	switch (resp.type) {
		case "update":
			break;

		case "query":
			break;

		default:
			const invalidResp: never = resp;
			throw new Error(`received invalid response from search worker: ${invalidResp}`);
	}
};
searchWorker.postMessage({
	type: "query",
	view: { truth: { base_url: "https://lms.pps.net" }, viewer: { User: 89090000000116506 } },
	query: { text: "" }
});

export default app;
