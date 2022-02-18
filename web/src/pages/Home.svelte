<script lang="ts">
	import { makeCanvasRequest, view } from "../view";
	import FaArrowRight from "svelte-icons/fa/FaArrowRight.svelte";
	import { Link } from "svelte-navigator";

	// FIXME: fetch courses from Oil

	let courses: Canvas.Course[] = [];
	// this could theoretically cause a race condition, but it's probably fine
	// since very few users will switch views anyway
	$: makeCanvasRequest($view, "/api/v1").then(data => courses = data);
</script>

<h1>Courses</h1>

{#each courses as course}
	<Link to={`/courses/${course.id}`}>
		<div class="card">
			<div class="card-content">
				<h2>{course.name}</h2>
				<p>{course.course_code}</p>
			</div>

			<span class="card-arrow"><FaArrowRight /></span>
		</div>
	</Link>
{/each}

<style lang="scss">
	.card {
		border-radius: var(--size-4);

		margin: var(--size-5) var(--size-2);
		padding: var(--size-4);

		display: flex;
		flex-direction: row;

		background-color: var(--color-blue);
        color: var(--color-grey-900);

		.card-content {
			flex-grow: 1;

			h2 {
				margin: 0;
				margin-bottom: 0.25em;
			}

			p {
				margin: 0;
			}
		}

		.card-arrow {
			width: var(--scale-4);

			// center icon
			display: flex;
			flex-direction: column;
			justify-content: center;
		}
	}
</style>
