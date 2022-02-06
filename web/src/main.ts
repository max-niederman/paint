import App from "./App.svelte";
import type * as Search from "./search/worker";
import "./styles/global.scss";

const app = new App({
	target: document.getElementById("app")
});

const searchWorker: Search.SearchWorker = new Worker("/search/worker.js");

searchWorker.onmessage = async (msg) => {
	const resp: Search.Response = msg.data;

	// FIXME: remove this
	console.log(`received response from search worker of type '${resp.type}'`);
	console.log(resp);

	switch (resp.type) {
		case "update":
			searchWorker.postMessage({
				type: "save"
			});
			// searchWorker.postMessage({
			// 	type: "query",
			// 	view: {
			// 		truth: { base_url: "https://lms.pps.net" },
			// 		viewer: { User: 89090000000116506n }
			// 	},
			// 	query: {
			// 		count: 10
			// 	}
			// });
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
		view: {
			truth: { base_url: "https://lms.pps.net" },
			viewer: { User: 89090000000116506n }
		},
		query: {
			sorted: false,
			count
		}
	});
// searchWorker.postMessage({
// 	type: "update",
// 	view: { truth: { base_url: "https://lms.pps.net" }, viewer: { User: 89090000000116506n } },
// 	since: "2022-01-01T00:00:00Z"
// });
