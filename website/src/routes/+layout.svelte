<script lang="ts">
	import "../app.css";
	import Nav from "$lib/components/Nav.svelte";
	import ErrorBar from "$lib/components/ErrorBar.svelte";
	import PlaygroundDock from "$lib/components/PlaygroundDock.svelte";
	import CommandPalette from "$lib/components/CommandPalette.svelte";
	import { doux } from "$lib/doux";
	import { stopScope, resetActiveEditor } from "$lib/scope";
	import { palette } from "$lib/palette.svelte";

	let { children } = $props();

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === "Escape") {
			if (palette.open) return;
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
<PlaygroundDock />
<CommandPalette />
<ErrorBar />
