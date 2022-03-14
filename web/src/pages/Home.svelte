<script lang="ts">
	import { makeViewRequest, view } from "../view";
	import FaArrowRight from "svelte-icons/fa/FaArrowRight.svelte";
	import { Link } from "svelte-navigator";

	// FIXME: fetch courses from Oil

	let courses: Canvas.Course[] = [];

	// NOTE: for whatever reason using a reactive statement sends two requests.
	makeViewRequest.subscribe(async (request) =>
		request(`/courses`)
			.then((resp) => resp.json())
			.then((data) => courses = data)
	);

	$: favoriteCourses = courses.filter((course) => course.is_favorite);
</script>

<h1>Courses</h1>

{#each favoriteCourses as course}
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
