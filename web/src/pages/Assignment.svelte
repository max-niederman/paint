<script lang="ts">
	import { makeViewRequest, view } from "../view";
	import { Link } from "svelte-navigator";
	import sanitizeHtml from "sanitize-html";
	import * as nav from "../components/Nav.svelte";

	export let courseId: number;
	export let id: number;

	let course: Canvas.Course = null;
	makeViewRequest.subscribe(async (request) => {
		course = await (await request(`/courses/${courseId}`)).json();
	});

	let assignment: Canvas.Assignment = null;
	makeViewRequest.subscribe(async (request) => {
		assignment = await (await request(`/courses/${courseId}/assignments/${id}`)).json();
	});

	nav.update.set(null);
	$: nav.upstreamURL.set(`https://${$view.canvas_domain}/courses/${courseId}/assignments/${id}`);

	function formatSubmissionTypes(types: Canvas.SubmissionType[]) {
		const typeNames: Record<Canvas.SubmissionType, string> = {
			discussion_topic: "Discussion Topic",
			online_quiz: "Online Quiz",
			on_paper: "On Paper",
			none: "None",
			external_tool: "External Tool",
			online_text_entry: "Online Text Entry",
			online_url: "Online URL",
			online_upload: "Online Upload",
			media_recording: "Media Recording",
			student_annotation: "Student Annotation",
			basic_lti_launch: "LTI Tool",
			not_graded: "Ungraded"
		};

		if (types.length === 0) {
			return "None";
		} else if (types.length === 1) {
			return typeNames[types[0]];
		} else if (types.length === 2) {
			return `${typeNames[types[0]]} or ${typeNames[types[1]]}`;
		} else {
			let formatted = "";

			types.slice(0, -1).forEach((type) => {
				formatted += `${typeNames[type]}, `;
			});

			formatted += `or ${typeNames[types[types.length - 1]]}`;

			return formatted;
		}
	}

	function displayISOTime(iso8601: string) {
		const date = new Date(iso8601);

		if (date.getTime() === 0) {
			return "N/A";
		}

		return date.toLocaleString();
	}
</script>

{#if assignment !== null}
	<h1>{assignment.name}</h1>
	<div class="subheading">
		In
		<span class="course-link">
			<Link to={`/courses/${courseId}`}>
				{course !== null ? course.name : `Course #${courseId}`}
			</Link>
		</span>
	</div>

	{#if assignment.description}
		<h2>Description</h2>
		<div class="description">{@html sanitizeHtml(assignment.description)}</div>
	{/if}

	<h2>Timeline</h2>
	<p><b>Created:</b> {displayISOTime(assignment.created_at)}</p>
	<p><b>Last Updated:</b> {displayISOTime(assignment.updated_at)}</p>
	<p><b>Unlocks:</b> {displayISOTime(assignment.due_at)}</p>
	<p><b>Due:</b> {displayISOTime(assignment.due_at)}</p>
	<p><b>Locks:</b> {displayISOTime(assignment.lock_at)}</p>

	<h2>Submission</h2>
	<p><b>Type:</b> {formatSubmissionTypes(assignment.submission_types)}</p>
{:else}
	<h1>Assignment #{id}</h1>
	<div class="subheading">
		In
		<span class="course-link">
			<Link to={`/courses/${courseId}`}>
				{course !== null ? course.name : `Course #${courseId}`}
			</Link>
		</span>
	</div>
	<p>Loading...</p>
{/if}

<style lang="scss">
	.subheading {
		font-style: italic;

		.course-link {
			color: var(--color-blue-300);
			text-decoration: dashed underline;
		}
	}

	.description {
		:global(a) {
			color: var(--color-blue-300);
			text-decoration: dashed underline;
		}
	}
</style>
