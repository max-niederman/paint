<script lang="ts">
	import { authToken, makeAuthedRequest } from "../auth";
	import { updateViews } from "../view";
	import Button from "../components/Button.svelte";
	import * as nav from "../components/Nav.svelte";

	nav.clear();

	let formValues = {
		canvasDomain: "",
		canvasToken: ""
	};

	$: onSubmit = async () => {
		// TODO: proper error handling

		const newView: Oil.NewView = {
			name: "Default (created during onboarding)",
			canvas_domain: formValues.canvasDomain,
			canvas_access_token: formValues.canvasToken
		};

		await $makeAuthedRequest("/views", {
			method: "POST",
			headers: {
				"Content-Type": "application/json"
			},
			body: JSON.stringify(newView)
		});

		updateViews($authToken);
	};
</script>

<h1>Onboarding</h1>

<p>In order to use Paint, you'll need some information about your school's Canvas.</p>

<form on:submit|preventDefault={onSubmit}>
	<label>
		<h2>Canvas Domain</h2>

		<p>The domain you use to access Canvas.</p>

		<input type="text" placeholder="canvas.instructure.com" bind:value={formValues.canvasDomain} />
	</label>

	<label>
		<h2>Canvas Token</h2>

		<p>You can get a token by following these steps:</p>

		<ol>
			<li>Login to Canvas and click on the account icon in the sidebar.</li>
			<li>Open "Settings" and scroll down until you see the "Approved Integrations" section.</li>
			<li>
				Press the "New Access Token" button and follow the directions.
				<ul>
					<li>
						Make sure <b>NOT</b> to add an expiration date. If you do, Paint will stop working once the token expires.
					</li>
					<li>Put in whatever you want for "Purpose." It doesn't matter.</li>
				</ul>
			</li>
			<li>Paste the token into the field below and press the "Next" button.</li>
		</ol>

		<input type="text" placeholder="Paste Token Here" bind:value={formValues.canvasToken} />
	</label>

	<Button type="submit">Continue</Button>
</form>

<style lang="scss">
	form {
		h2 {
			margin-bottom: var(--size-2);
		}
		input {
			border: none;
			border-radius: var(--size-2);
			font-size: var(--scale-0);
			width: 240px;
			padding: 0.5em;
		}

		label {
			display: block;
			margin: var(--size-6) 0;
		}
	}
</style>
