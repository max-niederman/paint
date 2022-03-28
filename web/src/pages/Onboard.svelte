<script lang="ts">
    import { authToken, makeAuthedRequest } from "../auth";
    import { updateViews, views } from "../view";
    import Button from "../components/Button.svelte";

    let canvasToken: string = "";
</script>

<h1>Onboarding</h1>

<p>In order to use Paint, you'll need to get a Canvas API key.</p>

<p>You can get one by following these steps:</p>

<ol>
    <li>Login to Canvas and click on the account icon in the sidebar.</li>
    <li>Open "Settings" and scroll down until you see the "Approved Integrations" section.</li>
    <li>
        Press the "New Access Token" button and follow the directions.
        <ul>
            <li>Make sure <b>NOT</b> to add an expiration date. If you do, Paint will stop working once the token expires.</li>
            <li>Put in whatever you want for "Purpose." It doesn't matter.</li>
        </ul>
    </li>
    <li>Paste the token into the field below and press the "Next" button.</li>
</ol>

<form>
    <label>
        <h2>Canvas Token</h2>
        <input type="text" placeholder="Paste Token Here" bind:value={canvasToken} />
    </label>

    <Button type="submit">Continue</Button>
</form>


<style lang="scss">
    form {
        input {
            border: 2px solid;
            border-radius: 4px;
            font-size: 1em;
            min-width: 30ch;
            padding: 0.5rem;
        }

        label {
            display: block;
        }

    }
</style>