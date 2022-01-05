<script lang="ts">
	import { Router, Route, Link } from "svelte-navigator";
	import Nav from "./components/Nav.svelte";
	import SearchPage from "./pages/Search.svelte";
	import Button from "./components/Button.svelte";
	import HomePage from "./pages/Home.svelte";
</script>

<Router>
	<Nav />

	<div class="container">
		<div class="page">
			<Route path="/">
				<HomePage />
			</Route>

			<Route path="/search">
				<SearchPage />
			</Route>

			<Route path="/**">
				<main>
					<h1>404 Not Found</h1>
					<p>It looks like you're lost.</p>
					<Link to="/"><Button>Back Home</Button></Link>
				</main>
			</Route>
		</div>
	</div>
</Router>

<style lang="scss">
	.container {
		// pad the edges of mobile viewports
		padding: 0 var(--size-4);

		.page {
			margin: auto;
			margin-top: var(--size-8);

			width: 85ch;
			max-width: 100%;
		}
	}
</style>
