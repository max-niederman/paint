<script lang="ts">
	import { view as selected, view, views } from "../view";

    let selectedIdx = null;

    $: {
        if (selectedIdx === null) {
            selectedIdx = $views.findIndex((v) => v.id === $selected.id);
        } else if ($selected !== $views[selectedIdx]) {
            $selected = $views[selectedIdx];
        }
    };
</script>

{#if $views !== null && $views.length > 1}
	<select bind:value={selectedIdx}>
		{#each $views as view, idx}
			<option value={idx}>{view.name}</option>
		{/each}
	</select>
{/if}
