<script lang="ts">
	import { page } from '$app/state';
	import { ChevronDown, ChevronUp } from 'lucide-svelte';
	import CodeEditor from './CodeEditor.svelte';
	import Scope from './Scope.svelte';

	let expanded = $state(false);
	let height = $state(0);

	$effect(() => {
		const onReference = page.url.pathname.startsWith('/reference');
		const mobile = window.matchMedia('(max-width: 768px)').matches;
		expanded = onReference && !mobile;
	});

	$effect(() => {
		document.documentElement.style.setProperty('--dock-h', `${height}px`);
		return () => document.documentElement.style.setProperty('--dock-h', '0px');
	});
</script>

<div class="dock" class:collapsed={!expanded} bind:clientHeight={height}>
	<div class="dock-editor">
		<CodeEditor code={'/sound/sine'} rows={2} />
	</div>
	<div class="dock-scope">
		<Scope />
	</div>
	<button
		class="dock-toggle"
		onclick={() => (expanded = !expanded)}
		title={expanded ? 'collapse playground' : 'expand playground'}
	>
		{#if expanded}
			<ChevronDown size={14} />
		{:else}
			<ChevronUp size={14} />
		{/if}
	</button>
</div>

<style>
	.dock {
		position: fixed;
		left: 0;
		right: 0;
		bottom: var(--tabbar-h);
		z-index: 90;
		display: flex;
		align-items: stretch;
		gap: 8px;
		padding: 8px 16px;
		border-top: 1px solid var(--hairline-strong);
		background: var(--bg);
	}

	.dock-editor {
		flex: 1;
		min-width: 0;
		max-width: 60ch;
	}

	.dock :global(.repl) {
		margin: 0;
	}

	.dock-scope {
		flex: 1;
		min-width: 0;
		border: 1px solid var(--hairline);
	}

	.dock-scope :global(.scope) {
		height: 100%;
	}

	.dock-toggle {
		width: 32px;
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 0;
		color: var(--muted);
	}

	.dock.collapsed {
		padding: 4px 16px;
	}

	.dock.collapsed .dock-editor {
		display: none;
	}

	.dock.collapsed .dock-scope {
		height: 26px;
		border: none;
	}

	.dock.collapsed .dock-toggle {
		border: none;
		background: none;
	}
</style>
