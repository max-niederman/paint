<script lang="ts">
	import FaHome from "svelte-icons/fa/FaHome.svelte";
	import MdSettings from "svelte-icons/md/MdSettings.svelte";
	import { Link } from "svelte-navigator";
</script>

<nav>
	<Link to="/"><div class="icon"><FaHome /></div></Link>
	<Link to="/settings"><div class="icon"><MdSettings /></div></Link>
</nav>

<style lang="scss">
	$icon-size: var(--size-7);

	nav {
		height: var(--size-8);

		display: flex;
		flex-direction: row;
		justify-content: space-between;

		.icon {
			margin: var(--size-5);
			height: $icon-size;
			width: $icon-size;
            color: var(--color-foreground);
		}
	}
</style>
