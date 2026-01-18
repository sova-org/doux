<script lang="ts">
	import { doux } from '$lib/doux';
	import { Home, FileText, LifeBuoy, Terminal } from 'lucide-svelte';
	import Scope from './Scope.svelte';

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
	<a href="/" class="nav-title"><h1>Doux</h1></a>
	<div class="nav-links">
		<a href="/" class="nav-link"><Home size={16} /> Home</a>
		<a href="/reference" class="nav-link"><FileText size={16} /> Reference</a>
		<a href="/native" class="nav-link"><Terminal size={16} /> Native</a>
		<a href="/support" class="nav-link"><LifeBuoy size={16} /> Support</a>
	</div>
	<div class="nav-scope">
		<Scope />
	</div>
	<button
		class="mic-btn"
		class:mic-enabled={micEnabled}
		disabled={micLoading}
		onclick={toggleMic}
	>
		{micLoading ? '...' : '🎤 Microphone'}
	</button>
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
	}

	.nav-title {
		text-decoration: none;
		margin-right: 16px;
	}

	.nav-title h1 {
		margin: 0;
	}

	.nav-link {
		display: flex;
		align-items: center;
		gap: 6px;
		text-decoration: none;
		color: #666;
	}

	.nav-link:hover {
		color: #000;
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
		color: #666;
		text-decoration: none;
	}

	.nav-tab:hover {
		color: #000;
		background: #f5f5f5;
	}

	@media (max-width: 768px) {
		.nav-links {
			display: none;
		}

		.nav-tabs {
			display: flex;
			position: fixed;
			bottom: 0;
			left: 0;
			right: 0;
			height: 48px;
			background: #fff;
			border-top: 1px solid #ccc;
			z-index: 100;
		}
	}
</style>
