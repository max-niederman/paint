<script lang="ts">
	import { Router, Route, Link } from "svelte-navigator";
	import Nav from "./components/Nav.svelte";
	import Button from "./components/Button.svelte";
	import SearchPage from "./pages/Search.svelte";
	import HomePage from "./pages/Home.svelte";
	import CoursePage from "./pages/Course.svelte";
	import OnboardPage from "./pages/Onboard.svelte";
	import { isLoading as isAuthLoading, isAuthenticated, createAuth } from "./auth";
	import { view, views } from "./view";

	const auth = createAuth();

	$: {
		if (!$isAuthLoading && !$isAuthenticated) {
			auth.login();
		}
	}
</script>

{#if !$isAuthLoading}
	{#if $isAuthenticated}
		<Router>
			<Nav />

			<div class="container">
				<div class="page">
					{#if $view}
						<Route path="/">
							<HomePage />
						</Route>

						<Route path="/search">
							<SearchPage />
						</Route>

						<Route path="/courses/:id" let:params>
							<CoursePage id={parseInt(params.id)} />
						</Route>
					{:else}
						<Route path="/**">
							<OnboardPage />
						</Route>
					{/if}


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
	{:else}
		<div class="container">
			<div class="page">
				<main class="login">
					<h1>Not Logged In</h1>
					<p>Redirecting...</p>
					<p>
						If you aren't redirected within a few seconds you can
						<span class="login-link" on:click={() => auth.login()}>login manually.</span>
					</p>
				</main>
			</div>
		</div>
	{/if}
{:else}
	<div class="container">
		<div class="page">
			<main class="login">
				<h1>Loading...</h1>
			</main>
		</div>
	</div>
{/if}

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

	.login-link {
		color: var(--color-blue);
		cursor: pointer;
	}
</style>
