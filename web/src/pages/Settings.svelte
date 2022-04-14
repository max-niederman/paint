<script lang="ts">
	import { updateViews, views, view as currentView } from "../view";
	import { authToken, getAuth, makeAuthedRequest } from "../auth";
	import Button from "../components/Button.svelte";
	import * as nav from "../components/Nav.svelte";
	import FaTrash from "svelte-icons/fa/FaTrash.svelte";

	nav.clear();

	let viewActions = (_: Oil.View) => ({
		delete: () => console.error("attempted to delete view before views were loaded")
	});
	$: viewActions = (view: Oil.View) => ({
		delete: async () => {
			await $makeAuthedRequest(`/views/${view.id}`, { method: "DELETE" });
			updateViews($authToken);
			if (view.id === $currentView.id) {
				currentView.set($views[0] ?? null);
			}
		}
	});
</script>

<h1>Settings</h1>

<h2>Views</h2>
<table>
	<thead>
		<tr>
			<th>Name</th>
			<th>Canvas Domain</th>
			<th>Actions</th>
		</tr>
	</thead>
	<tbody>
		{#each $views as view}
			<tr>
				<td>{view.name}</td>
				<td>{view.canvas_domain}</td>
				<td>
					<div class="action" on:click={viewActions(view).delete}><FaTrash /></div>
				</td>
			</tr>
		{/each}
	</tbody>
</table>

<h2>Account Actions</h2>

<Button on:click={getAuth().logout}>Log Out</Button>

<style lang="scss">
	.action {
		cursor: pointer;
	}

	table {
		width: 100%;

		thead th {
			text-align: left;

			&:nth-child(1) {
				width: 30%;
			}

			&:nth-child(2) {
				width: 40%;
			}
		}

		td {
			padding: 0.5em 0;
		}

		.action {
			height: 1.15em;
			width: 1.15em;
		}
	}
</style>
