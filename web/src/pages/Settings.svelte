<script lang="ts">
	import { getAuth } from "../auth";
	import Button from "../components/Button.svelte";
</script>

<h1>Settings</h1>

<!-- TODO: add view list -->

<Button on:click={getAuth().logout}>Log Out</Button>
