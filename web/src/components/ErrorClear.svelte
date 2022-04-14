<!-- NOTE: this is necessary because we can't use svelte-navigator's `useLocation` outside of a router -->
<script lang="ts">
	import { error } from "../error";
	import { useLocation } from "svelte-navigator";

	const location = useLocation();

	location.subscribe(() => error.set(null));
</script>
