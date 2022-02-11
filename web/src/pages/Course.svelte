<script lang="ts">
import { onMount } from "svelte";
import { navigate } from "svelte-navigator";


    export let id: number;

    // FIXME: use user's canvas base url
    onMount(() => {
        window.location.assign(`https://lms.pps.net/courses/${id}`);
        navigate(-1);
    })
</script>

<!-- TODO: write in-Paint course page -->
<h1>Course #{id}</h1>
<p>Redirecting...</p>