<script lang="ts">
	import { makeViewRequest, view } from "../view";
	import { Link } from "svelte-navigator";
	import Card from "../components/Card.svelte";
	import FaLock from "svelte-icons/fa/FaLock.svelte";
	import FaRegSquare from "svelte-icons/fa/FaRegSquare.svelte";
	import FaRegMinusSquare from "svelte-icons/fa/FaRegMinusSquare.svelte";
	import FaRegCheckSquare from "svelte-icons/fa/FaRegCheckSquare.svelte";

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
		.sort((a, b) => new Date(a.due_at).getTime() - new Date(b.due_at).getTime())
		.sort((a, b) => (a.has_submitted_submissions ? 1 : 0) - (b.has_submitted_submissions ? 1 : 0))
		.sort((a, b) => (a.locked_for_user ? 1 : 0) - (b.locked_for_user ? 1 : 0));

	function displayISODate(iso: string) {
		const date = new Date(iso);

		if (date.getTime() === 0) {
			return "N/A";
		}

		return date.toLocaleDateString();
	}
</script>

{#if course !== null}
	<h1>{course.name}</h1>

	<h2>Assignments</h2>

	{#each sortedAssignments as assignment}
		<Link to={`/courses/${id}/assignments/${assignment.id}`}>
			<Card>
				<div class="card-content">
					<h3>{assignment.name}</h3>
					<p>
						Due: {displayISODate(assignment.due_at)}, Locks: {displayISODate(assignment.lock_at)}, Submitted: {assignment.has_submitted_submissions
							? "Yes"
							: "No"}, Locked: {assignment.locked_for_user ? "Yes" : "No"}
					</p>
				</div>

				<svelte:fragment slot="right">
					<span class="card-icon">
						{#if assignment.locked_for_user}
							<FaLock />
						{:else if assignment.has_submitted_submissions}
							<FaRegCheckSquare />
						{:else if assignment.submission_types.length === 0 || assignment.submission_types.includes("not_graded")}
							<FaRegMinusSquare />
						{:else}
							<FaRegSquare />
						{/if}
					</span>
				</svelte:fragment>
			</Card>
		</Link>
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

	.card-icon {
		height: 100%;
		width: var(--scale-4);

		margin: 0 var(--size-2);

		// center icon
		display: flex;
		flex-direction: column;
		justify-content: center;
	}
</style>
