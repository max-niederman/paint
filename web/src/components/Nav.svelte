<script lang="ts">
	import FaHome from "svelte-icons/fa/FaHome.svelte";
	import MdSettings from "svelte-icons/md/MdSettings.svelte";
	import { Link } from "svelte-navigator";
	import ViewSelector from "./ViewSelector.svelte";
</script>

<nav>
	<div>
		<Link to="/"><div class="icon"><FaHome /></div></Link>
	</div>
	<div>
		<ViewSelector />
		<div class="spacer" />
		<Link to="/settings"><div class="icon"><MdSettings /></div></Link>
	</div>
</nav>

<style lang="scss">
	$icon-size: var(--size-7);

	nav {
		padding: var(--size-4);

		display: flex;
		flex-direction: row;
		justify-content: space-between;
		align-items: stretch;

		// each div serves as a section, either left- or right- aligned
		div {
			display: flex;
			flex-direction: row;
			align-items: center;
		}
		.icon {
			height: $icon-size;
			width: $icon-size;
			color: var(--color-foreground);
		}

		.spacer {
			width: var(--size-3);
		}
	}
</style>
