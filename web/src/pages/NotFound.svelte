<script>
	import Button from "../components/Button.svelte";
	import { Link } from "svelte-navigator";
	import { error } from "../error";
</script>

<main>
	<h1>404 Not Found</h1>
	<p>{$error["help"] ?? "It looks like you're lost."}</p>
	<Link to="/"><Button>Back Home</Button></Link>
</main>
