<script lang="ts">
	import "../app.css";
	import Nav from "$lib/components/Nav.svelte";
	import { doux } from "$lib/doux";
	import { stopScope, resetActiveEditor } from "$lib/scope";

	let { children } = $props();

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === "Escape") {
			resetActiveEditor();
			stopScope();
			doux.hush();
		}
	}
</script>

<svelte:window onkeydown={handleKeydown} />

<Nav />
<div class="layout">
	{@render children()}
</div>
