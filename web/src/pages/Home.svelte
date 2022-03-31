<script lang="ts">
	import { makeViewRequest, view } from "../view";
	import FaArrowRight from "svelte-icons/fa/FaArrowRight.svelte";
	import { Link } from "svelte-navigator";
	import Card from "../components/Card.svelte";
	import * as nav from "../components/Nav.svelte";

	let courses: Canvas.Course[] = null;
	makeViewRequest.subscribe(async (request) => {
		courses = await (await request(`/courses`)).json();

		if (courses.length === 0) {
			courses = null;
			await request(`/courses/update`, { method: "POST" });
			courses = await (await request(`/courses`)).json();
		}
	});

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
	
	$: nav.upstreamURL.set(`https://${$view.canvas_domain}`);
	$: nav.update.set(async () => {
		displayedCourses = null;
		await $makeViewRequest(`/courses/update`, { method: "POST" });
		courses = await (await $makeViewRequest(`/courses`)).json();
	})

	$: console.log("courses", courses)
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

				<span class="card-arrow" slot="right">
					<FaArrowRight />
				</span>
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
	<p>Loading courses...</p>
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

	.card-arrow {
		height: 100%;
		width: var(--scale-4);

		// center icon
		display: flex;
		flex-direction: column;
		justify-content: center;
	}
</style>
