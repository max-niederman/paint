<script lang="ts">
	import { makeViewRequest, view } from "../view";
	import FaArrowRight from "svelte-icons/fa/FaArrowRight.svelte";
	import { Link } from "svelte-navigator";
	import Card from "../components/Card.svelte";

	let courses: Canvas.Course[] = null;
	makeViewRequest.subscribe(async (request) =>
		request(`/courses`)
			.then((resp) => resp.json())
			.then(async (data) => {
				if (data.length > 0) {
					courses = data;
				} else {
					await request(`/courses/update`, { method: "POST" });

					const resp = await request(`/courses`);
					courses = await resp.json();
				}
			})
	);

	let displayedCourses: Canvas.Course[] = null;
	$: {
		if (courses !== null) {
			let favorites = courses.filter((course) => course.is_favorite);
			if (favorites.length !== 0) {
				displayedCourses = favorites;
			} else {
				displayedCourses = courses;
			}
		}
	}
</script>

<h1>Courses</h1>

{#if displayedCourses !== null}
	{#each displayedCourses as course}
		<Link to={`/courses/${course.id}`}>
			<Card>
				<div class="card-content">
					<h2>{course.name}</h2>
					<p>{course.course_code}</p>
				</div>
			</Card>
		</Link>
	{:else}
		<p>Well... this is awkward; you have no courses. This leaves two possibilities:</p>
		<p />
		<ol>
			<li>I messed up.</li>
			<li>You're trying to use Paint with an empty Canvas account.</li>
		</ol>
		<p>Given how weird #2 is, this is probably the former.</p>
	{/each}
{:else}
	<p>Loading...</p>
{/if}

<style lang="scss">
	.card-content {
		h2 {
			margin: 0;
			margin-bottom: 0.25em;
		}

		p {
			margin: 0;
		}
	}
</style>
