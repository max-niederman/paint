<script lang="ts">
	import { Router, Route, Link, useLocation } from "svelte-navigator";
	import Nav from "./components/Nav.svelte";
	import ErrorClear from "./components/ErrorClear.svelte";
	import SettingsPage from "./pages/Settings.svelte";
	import HomePage from "./pages/Home.svelte";
	import CoursePage from "./pages/Course.svelte";
	import OnboardPage from "./pages/Onboard.svelte";
	import AssignmentPage from "./pages/Assignment.svelte";
	import NotFoundPage from "./pages/NotFound.svelte";
	import { isLoading as isAuthLoading, isAuthenticated, createAuth } from "./auth";
	import { views } from "./view";
	import { error } from "./error";

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

			<ErrorClear />

			<div class="container">
				<div class="page">
					{#if $error === null}
						{#if $views?.length === 0}
							<Route path="/**">
								<OnboardPage />
							</Route>
						{:else}
							<Route path="/">
								<HomePage />
							</Route>

							<Route path="/settings">
								<SettingsPage />
							</Route>

							<Route path="/courses/:id" let:params>
								<CoursePage id={parseInt(params.id)} />
							</Route>

							<Route path="/courses/:courseId/assignments/:id" let:params>
								<AssignmentPage courseId={parseInt(params.courseId)} id={parseInt(params.id)} />
							</Route>
						{/if}

						<Route path="/**">
							<NotFoundPage />
						</Route>
					{:else if $error?.type === "not_found"}
						<NotFoundPage />
					{:else if $error?.type === "unknown_http_status"}
						<main>
							<h1>Unknown HTTP Status Error</h1>
							<p>Please try again later. Sorry for the inconvenience.</p>

							<details>
								<summary>Error Details</summary>
								<p>Status: {$error.status}</p>
								<code>{JSON.stringify($error.resp, undefined, 4)}</code>
							</details>
						</main>
					{:else if $error?.type === "server_error"}
						<main>
							<h1>Internal Server Error</h1>
							<p>Please try again later. Sorry for the inconvenience.</p>

							<details>
								<summary>Error Details</summary>
								<code>{JSON.stringify($error.body, undefined, 4)}</code>
							</details>
						</main>
					{/if}
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
