<script lang="ts">
	import FaArrowRight from "svelte-icons/fa/FaArrowRight.svelte";
	import { Link } from "svelte-navigator";

	// FIXME: fetch courses from Oil
	const courses: Partial<Canvas.Course>[] = [
		{
            id: 81546,
			name: "English 1-2",
			course_code: "1560H290-Robins"
		},
		{
            id: 81547,
			name: "World History",
			course_code: "2000H120-Smith"
		}
	];
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
