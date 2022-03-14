<script lang="ts">
    import { makeViewRequest, view } from "../view";
    import { navigate } from "svelte-navigator";

    export let id: number;

    let course: Canvas.Course = null;
	makeViewRequest.subscribe(async (request) =>
		request(`/courses/${id}`)
			.then((resp) => resp.json())
			.then((data) => course = data)
	);
</script>

{#if course !== null}
    <h1>{course.name}</h1>
{:else}
    <h1>Course #{id}</h1>
    <p>Loading...</p>
{/if}