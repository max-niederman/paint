<script lang="ts">
    import FaArrowRight from "svelte-icons/fa/FaArrowRight.svelte";
</script>

<div class="card">

    <div class="card-content">
        <slot />
    </div>

	<span class="card-icon">
        <slot name="icon">
            <FaArrowRight />
        </slot>
    </span>
</div>

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
		}

		.card-icon {
			width: var(--scale-4);

			// center icon
			display: flex;
			flex-direction: column;
			justify-content: center;
		}
	}
</style>
