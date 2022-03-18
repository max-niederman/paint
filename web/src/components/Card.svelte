<script lang="ts">
    import FaArrowRight from "svelte-icons/fa/FaArrowRight.svelte";
</script>

<div class="card">
    <div class="main-content">
        <slot />
    </div>

	<div class="right-content">
		<slot name="right" />

		<span class="icon">
			<slot name="icon">
				<FaArrowRight />
			</slot>
		</span>
	</div>
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

		.main-content {
			flex-grow: 1;
		}

		.icon {
			width: var(--scale-4);

			// center icon
			display: flex;
			flex-direction: column;
			justify-content: center;
		}
	}
</style>
