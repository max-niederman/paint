<script lang="ts">
	import { makeCanvasRequest, makeCanvasRequestPaginated, view } from "../view";
	import FaArrowRight from "svelte-icons/fa/FaArrowRight.svelte";
	import { Link } from "svelte-navigator";

	// FIXME: fetch courses from Oil

	let courses: Canvas.Course[] = [];
	// FIXME: currently if the user is enrolled in more classes than can be listed in one page, those classes will not be visible
	//  NOTE: this could theoretically cause a race condition, but it's probably fine
	//        since very few users will switch views anyway
	$: makeCanvasRequestPaginated<Canvas.Course>($view, "/api/v1/courses?enrollment_state=active").then(data => courses = data);

	$: visibleCourses = courses.filter((course) => course.overridden_course_visibility !== undefined);
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
