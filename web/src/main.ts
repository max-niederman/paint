import App from "./App.svelte";
import type * as Search from "./search/worker";
import type { View } from "glaze-wasm";
import "./styles/global.scss";

const app = new App({
	target: document.getElementById("app")
});

const searchWorker: Search.SearchWorker = new Worker("/search/worker.js");
const view: View = {
	truth: { base_url: "https://lms.pps.net" },
	viewer: { User: 89090000000116506n }
};

searchWorker.onmessage = async (msg) => {
	const resp: Search.Response = msg.data;

	// FIXME: remove this
	console.log(`received response from search worker of type '${resp.type}'`);
	console.log(resp);

	switch (resp.type) {
		case "initialize":
			searchWorker.postMessage({ type: "update", view, since: "2022-01-01T00:00:00Z" });
			break;

		case "update":
			searchWorker.postMessage({ type: "save" });
			searchWorker.postMessage({
				type: "query",
				view,
				query: {
					limit: 10,
					sorted: false,
					targets: {
						course: true,
						assignment: false,
						submission: false
					},
				}
			});
			break;

		case "save":
			break;

		case "query":
			break;

		default:
			const invalidResp: never = resp;
			throw new Error(`received invalid response from search worker: ${invalidResp}`);
	}
};
(window as any).query = (count) =>
	searchWorker.postMessage({
		type: "query",
		view,
		query: {
			sorted: false,
			targets: {
				course: true,
				assignment: true,
				submission: true,
			},
			count
		}
	});
searchWorker.postMessage({ type: "initialize" });
