<script lang="ts">
	import { searchIndex, type SearchEntry } from '$lib/search/index';

	// anchor -> entry; commands win over same-named sections (e.g. "gain")
	const byAnchor = new Map<string, SearchEntry>();
	for (const e of searchIndex) {
		if (e.kind === 'param') continue;
		const existing = byAnchor.get(e.anchor);
		if (!existing || e.kind === 'command') byAnchor.set(e.anchor, e);
	}

	interface Card {
		entry: SearchEntry;
		x: number;
		y: number;
		above: boolean;
	}

	let card = $state<Card | null>(null);
	let currentLink: Element | null = null;

	function close() {
		currentLink = null;
		card = null;
	}

	function onMouseOver(e: MouseEvent) {
		const link = (e.target as Element).closest?.('a.xref');
		if (!link || link === currentLink) return;
		const anchor = (link.getAttribute('href') ?? '').split('#')[1];
		const entry = anchor ? byAnchor.get(anchor) : undefined;
		if (!entry) return;
		currentLink = link;
		const rect = link.getBoundingClientRect();
		const above = rect.bottom > window.innerHeight - 180;
		card = {
			entry,
			x: Math.max(8, Math.min(rect.left, window.innerWidth - 348)),
			y: above ? rect.top : rect.bottom,
			above
		};
	}

	function onMouseOut(e: MouseEvent) {
		if (!currentLink) return;
		if (e.relatedTarget instanceof Node && currentLink.contains(e.relatedTarget)) return;
		close();
	}
</script>

<svelte:document onmouseover={onMouseOver} onmouseout={onMouseOut} />
<svelte:window onscroll={close} />

{#if card}
	<div
		class="hover-card"
		style:left="{card.x}px"
		style:top="{card.y}px"
		style:transform={card.above ? 'translateY(calc(-100% - 6px))' : 'translateY(6px)'}
	>
		<div class="card-head">
			<span class="card-name">{card.entry.name}</span>
			{#if card.entry.mod}
				<span class="card-mod" title="accepts inline modulation">~mod</span>
			{/if}
			{#each card.entry.aliases as alias (alias)}
				<span class="card-alias">{alias}</span>
			{/each}
			<span class="label card-kind">{card.entry.kind}</span>
		</div>
		{#if card.entry.meta}
			<div class="card-meta">{card.entry.meta}</div>
		{/if}
		{#if card.entry.description}
			<p class="card-desc">{card.entry.description}</p>
		{/if}
	</div>
{/if}

<style>
	.hover-card {
		position: fixed;
		z-index: 250;
		width: 340px;
		pointer-events: none;
		background: var(--bg);
		border: 1px solid var(--hairline-strong);
		box-shadow: 0 4px 16px rgba(0, 0, 0, 0.12);
		padding: 10px 12px;
	}

	.card-head {
		display: flex;
		align-items: baseline;
		gap: 8px;
	}

	.card-name {
		font-family: var(--font-mono);
		font-size: 13px;
		font-weight: 600;
	}

	.card-mod {
		font-family: var(--font-mono);
		font-size: 10px;
		color: var(--accent);
		border: 1px solid color-mix(in srgb, var(--accent) 40%, transparent);
		padding: 0 5px;
		white-space: nowrap;
	}

	.card-alias {
		font-family: var(--font-mono);
		font-size: 12px;
		color: var(--faint);
	}

	.card-kind {
		margin-left: auto;
	}

	.card-meta {
		font-family: var(--font-mono);
		font-size: 11px;
		color: var(--muted);
		margin-top: 4px;
	}

	.card-desc {
		margin: 6px 0 0;
		font-size: 13px;
		color: var(--ink-soft);
	}
</style>
