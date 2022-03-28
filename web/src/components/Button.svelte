<script lang="ts">
	export let type: string = "button";
</script>

<button on:click {type}>
	<slot />
</button>

<style lang="scss">
	button {
        display: inline-block;

		padding: var(--size-4);
		text-align: center;
        text-decoration: none;

		font-family: var(--font-sans);
		font-size: inherit;
        font-weight: 800;

		cursor: pointer;

		background-color: var(--color-blue);
		color: var(--color-background);

		border: none;
        border-radius: var(--size-2);
	}
</style>
