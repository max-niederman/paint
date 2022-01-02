<script lang="ts">
	import { Router, Route } from "svelte-navigator";
</script>

<Router>
	<div class="background">
		<div class="page">
			<Route path="/">
				<h1>Search</h1>
				<!-- TODO: Search page -->
			</Route>
		</div>
	</div>
</Router>

<style lang="scss">
	.page {
		margin: auto;

		min-width: 640px;
		width: 85ch;
	}
</style>
