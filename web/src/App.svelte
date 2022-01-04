<script lang="ts">
	import { Router, Route } from "svelte-navigator";
	import Nav from "./components/Nav.svelte";
	import SearchPage from "./pages/Search.svelte";
</script>

<Router>
	<Nav />

	<div class="page">
		<Route path="/">
			<SearchPage />
		</Route>
	</div>
</Router>

<style lang="scss">
	.page {
		margin: auto;

		min-width: 640px;
		width: 85ch;
	}
</style>
