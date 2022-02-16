<script lang="ts">
	import FaHome from "svelte-icons/fa/FaHome.svelte";
	import MdSettings from "svelte-icons/md/MdSettings.svelte";
	import { Link } from "svelte-navigator";
	import ViewSelector from "./ViewSelector.svelte";
</script>

<nav>
	<Link to="/"><div class="icon"><FaHome /></div></Link>
	<div>
		<ViewSelector />
		<Link to="/settings"><div class="icon"><MdSettings /></div></Link>
	</div>
</nav>

<style lang="scss">
	$icon-size: var(--size-7);

	nav {
		height: var(--size-8);

		display: flex;
		flex-direction: row;
		justify-content: space-between;
		align-items: stretch;

		// `div` elements are used to seperate left-aligned elements from right-aligned elements
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
	}
</style>
