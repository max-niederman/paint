<script lang="ts">
	export let text: boolean = false;
</script>

{#if text}
	<input type="text" placeholder="Type to search..." />
{/if}
