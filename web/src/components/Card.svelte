<script lang="ts">
    import FaArrowRight from "svelte-icons/fa/FaArrowRight.svelte";
</script>

<div class="card">
    <div class="main-content">
        <slot />
    </div>

	<div class="right-content">
		<slot name="right" />
	</div>
</div>

<style lang="scss">
	.card {
		border-radius: var(--size-4);

		margin: var(--size-5) var(--size-2);
		padding: var(--size-4);

		display: flex;
		flex-direction: row;

		background-color: var(--color-green-300);
		color: var(--color-grey-900);

		.main-content {
			flex-grow: 1;
		}

        .right-content {
            display: flex;
            flex-direction: row;
            align-items: center;
        }
	}
</style>
