<script lang="ts">
	import { makeViewRequest, view } from "../view";
	import { Link, navigate } from "svelte-navigator";
	import Card from "../components/Card.svelte";
	import FaLock from "svelte-icons/fa/FaLock.svelte";
	import FaLockOpen from "svelte-icons/fa/FaLockOpen.svelte";

	export let id: number;

	let course: Canvas.Course = null;
	let assignments: Canvas.Assignment[] = [];
	makeViewRequest.subscribe(async (request) => {
		course = await (await request(`/courses/${id}`)).json();
		assignments = await (await request(`/courses/${id}/assignments`)).json();

        if (assignments.length === 0) {
            await request(`/courses/${id}/assignments/update`, { method: "POST" });
            assignments = await (await request(`/courses/${id}/assignments`)).json();
        }
	});

	$: sortedAssignments = assignments
		.sort((a, b) => new Date(a.due_at).getUTCSeconds() - new Date(b.due_at).getUTCSeconds())
		.sort((a, b) => (a.locked_for_user ? 1 : 0) - (b.locked_for_user ? 1 : 0))
		.sort((a, b) => (a.has_submitted_submissions ? 1 : 0) - (b.has_submitted_submissions ? 1 : 0));

	function displayISODate(iso: string) {
		const date = new Date(iso);
		return date.toLocaleDateString();
	}
</script>

{#if course !== null}
	<h1>{course.name}</h1>

	<h2>Assignments</h2>

	{#each sortedAssignments as assignment}
		<Card>
			<div class="card-content">
				<h3>{assignment.name}</h3>
				<p>
					Due: {displayISODate(assignment.due_at)}
					Locks: {displayISODate(assignment.lock_at)}
				</p>
			</div>

			<span slot="icon">
				{#if assignment.locked_for_user}
					<FaLock />
				{:else}
					<FaLockOpen />
				{/if}
			</span>
		</Card>
    {:else}
        <p>Assignments loading...</p>
	{/each}
{:else}
	<h1>Course #{id}</h1>
	<p>Loading...</p>
{/if}

<style lang="scss">
	.card-content {
		h3 {
			margin: 0;
			margin-bottom: 0.25em;
		}

		p {
			margin: 0;
		}
	}
</style>
