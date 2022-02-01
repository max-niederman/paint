import App from "./App.svelte";
import type * as Search from "./search/worker";
import "./styles/global.scss";

const app = new App({
	target: document.getElementById("app")
});

const searchWorker: Search.SearchWorker = new Worker("/search/worker.js");

searchWorker.onmessage = (msg) => {
	const resp: Search.Response = msg.data;

	console.log(`received response from search worker`);
	console.log(resp);

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
	type: "update",
	view: { truth: { base_url: "https://lms.pps.net" }, viewer: { User: 89090000000116506 } },
	since: "2022-01-01T00:00:00Z"
});
