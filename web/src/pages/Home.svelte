<script lang="ts">
	import { view } from "../view";
	import FaArrowRight from "svelte-icons/fa/FaArrowRight.svelte";
	import { Link } from "svelte-navigator";
	import { makeAuthedRequest } from "../auth";

	// FIXME: fetch courses from Oil

	let courses: Canvas.Course[] = [];
	$: $makeAuthedRequest(`/views/${$view.id}/courses`)
		.then((resp) => resp.json())
		.then((data) => (courses = data));

	$: visibleCourses = courses.filter((course) => course.overridden_course_visibility !== null);
</script>

<h1>Courses</h1>

{#each visibleCourses as course}
	<Link to={`/courses/${course.id}`}>
		<div class="card">
			<div class="card-content">
				<h2>{course.name}</h2>
				<p>{course.course_code}</p>
			</div>

			<span class="card-arrow"><FaArrowRight /></span>
		</div>
	</Link>
{:else}
	<p>Loading...</p>
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
