<script lang="ts">
    import { view } from "../view";
    import { navigate } from "svelte-navigator";

    export let id: number;

    $: {
        window.location.assign(`${$view.canvas_base_url}/courses/${id}`);
        navigate(-1);
    }
</script>

<!-- TODO: write in-Paint course page -->
<h1>Course #{id}</h1>
<p>Redirecting...</p>