<script lang="ts">
	import { onMount } from 'svelte';
	import { initScope } from '$lib/scope';

	let { class: className = '' }: { class?: string } = $props();
	let canvas: HTMLCanvasElement;

	onMount(() => {
		const cleanup = initScope(canvas);
		return cleanup;
	});
</script>

<div class="scope {className}">
	<canvas bind:this={canvas}></canvas>
</div>
