import initWasm, { SearchManager } from "glaze-wasm";
import type { View, Query } from "glaze-wasm";

let searchManager = await new SearchManager();
const view: View = {
	truth: { base_url: "https://lms.pps.net" },
	viewer: { User: 89090000000116506 }
};
searchManager.update(view);
console.log(searchManager.query(view, { text: "" }));

type Request =
	| {
			type: "initialize";
	  }
	| {
			type: "update";
			view: View;
	  }
	| {
			type: "query";
			view: View;
			query: Query;
	  };

onmessage = async (e) => {
    let req: Request = e.data;

    switch (req.type) {
        case "initialize":
            await initWasm();
            break;
        
        case "update":
            searchManager.update(req.view);
            break;

        case "query":
            break;

        default:
            const invalidReq: never = req;
            throw new Error(`Invalid request: ${invalidReq}`);
    }
};
