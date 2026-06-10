<script lang="ts">
	import { version } from '$app/environment';
	import { doux } from '$lib/doux';
	import { palette } from '$lib/palette.svelte';
	import { Home, FileText, LifeBuoy, Terminal, Mic, MicOff, Search } from 'lucide-svelte';

	let micEnabled = $state(false);
	let micLoading = $state(false);

	async function toggleMic() {
		if (micEnabled) {
			doux.disableMic();
			micEnabled = false;
		} else {
			micLoading = true;
			await doux.enableMic();
			micEnabled = true;
			micLoading = false;
		}
	}
</script>

<nav>
	<a href="/" class="nav-title"><h1>Doux</h1><span class="nav-version">v{version}</span></a>
	<div class="nav-links">
		<a href="/" class="nav-link"><Home size={16} /> Home</a>
		<a href="/reference" class="nav-link"><FileText size={16} /> Reference</a>
		<a href="/native" class="nav-link"><Terminal size={16} /> Native</a>
		<a href="/support" class="nav-link"><LifeBuoy size={16} /> Support</a>
	</div>
	<div class="nav-actions">
		<button class="search-btn" onclick={() => (palette.open = true)}>
			<Search size={14} />
			<span class="search-hint">search</span>
			<kbd>⌘K</kbd>
		</button>
		<button
			class="mic-btn"
			class:mic-enabled={micEnabled}
			disabled={micLoading}
			onclick={toggleMic}
			title={micEnabled ? 'disable microphone' : 'enable microphone'}
		>
			{#if micEnabled}
				<Mic size={14} />
			{:else}
				<MicOff size={14} />
			{/if}
		</button>
	</div>
</nav>

<div class="nav-tabs">
	<a href="/" class="nav-tab"><Home size={20} /></a>
	<a href="/reference" class="nav-tab"><FileText size={20} /></a>
	<a href="/native" class="nav-tab"><Terminal size={20} /></a>
	<a href="/support" class="nav-tab"><LifeBuoy size={20} /></a>
</div>

<style>
	.nav-links {
		display: flex;
		align-items: center;
		gap: 16px;
		flex: 1;
		margin-left: 24px;
	}

	.nav-title {
		text-decoration: none;
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		line-height: 1;
	}

	.nav-title h1 {
		margin: 0;
	}

	.nav-version {
		font-family: var(--font-mono);
		font-size: 10px;
		color: var(--faint);
	}

	.nav-link {
		display: flex;
		align-items: center;
		gap: 6px;
		text-decoration: none;
		color: var(--muted);
	}

	.nav-link:hover {
		color: var(--ink);
	}

	.nav-actions {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.search-btn {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 6px 10px;
		color: var(--muted);
	}

	.search-btn kbd {
		pointer-events: none;
	}

	.mic-btn {
		padding: 7px 10px;
	}

	.nav-tabs {
		display: none;
	}

	.nav-tab {
		flex: 1;
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 12px;
		color: var(--muted);
		text-decoration: none;
	}

	.nav-tab:hover {
		color: var(--ink);
		background: var(--code-bg);
	}

	@media (max-width: 768px) {
		.nav-links {
			display: none;
		}

		.search-hint,
		.search-btn kbd {
			display: none;
		}

		.nav-tabs {
			display: flex;
			position: fixed;
			bottom: 0;
			left: 0;
			right: 0;
			height: var(--tabbar-h);
			background: var(--bg);
			border-top: 1px solid var(--hairline-strong);
			z-index: 100;
		}
	}
</style>
