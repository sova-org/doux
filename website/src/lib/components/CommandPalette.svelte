<script lang="ts">
	import { goto, replaceState } from '$app/navigation';
	import { page } from '$app/state';
	import { palette } from '$lib/palette.svelte';
	import { searchIndex } from '$lib/search/index';
	import { search, type SearchResult } from '$lib/search/fuzzy';

	let query = $state('');
	let selected = $state(0);
	let input: HTMLInputElement | undefined = $state();

	let results = $derived(search(query, searchIndex));

	$effect(() => {
		if (palette.open) {
			query = '';
			selected = 0;
			input?.focus();
		}
	});

	function onWindowKeydown(e: KeyboardEvent) {
		if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
			e.preventDefault();
			palette.open = !palette.open;
		}
	}

	function onInputKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			e.stopPropagation();
			palette.open = false;
		} else if (e.key === 'ArrowDown') {
			e.preventDefault();
			selected = Math.min(selected + 1, results.length - 1);
		} else if (e.key === 'ArrowUp') {
			e.preventDefault();
			selected = Math.max(selected - 1, 0);
		} else if (e.key === 'Enter' && results[selected]) {
			select(results[selected]);
		} else {
			selected = 0;
		}
	}

	function select(r: SearchResult) {
		palette.open = false;
		const { anchor, kind } = r.entry;
		if (page.url.pathname.startsWith('/reference')) {
			// commands/params live in <details> whose id may collide with a section id
			const el =
				kind === 'section'
					? document.getElementById(anchor)
					: (document.querySelector(`details[id="${anchor}"]`) ?? document.getElementById(anchor));
			if (el instanceof HTMLDetailsElement) el.open = true;
			el?.scrollIntoView();
			replaceState(`#${anchor}`, {});
		} else {
			goto(`/reference#${anchor}`);
		}
	}

	function nameParts(r: SearchResult): { char: string; hit: boolean }[] {
		const hits = new Set(r.nameMatches);
		return [...r.entry.name].map((char, i) => ({ char, hit: hits.has(i) }));
	}
</script>

<svelte:window onkeydown={onWindowKeydown} />

{#if palette.open}
	<div
		class="palette-overlay"
		onclick={() => (palette.open = false)}
		onkeydown={() => {}}
		role="presentation"
	>
		<div class="palette" role="dialog" aria-label="search" onclick={(e) => e.stopPropagation()}>
			<input
				bind:this={input}
				bind:value={query}
				onkeydown={onInputKeydown}
				placeholder="search commands, params, sections…"
				spellcheck="false"
			/>
			{#if results.length}
				<ul class="results">
					{#each results as r, i (r.entry.kind + r.entry.slug + r.entry.name)}
						<li>
							<button
								class="result"
								class:selected={i === selected}
								onclick={() => select(r)}
								onmouseenter={() => (selected = i)}
							>
								<span class="result-name">
									{#each nameParts(r) as p}<span class:hit={p.hit}>{p.char}</span>{/each}
									{#each r.entry.aliases as alias}
										<span class="result-alias">{alias}</span>
									{/each}
								</span>
								<span class="result-desc">{r.entry.description}</span>
								<span class="result-where label">{r.entry.sectionTitle} · {r.entry.kind}</span>
							</button>
						</li>
					{/each}
				</ul>
			{:else if query.trim()}
				<div class="empty label">no matches</div>
			{/if}
		</div>
	</div>
{/if}

<style>
	.palette-overlay {
		position: fixed;
		inset: 0;
		z-index: 300;
		background: rgba(0, 0, 0, 0.15);
		display: flex;
		justify-content: center;
		align-items: flex-start;
		padding-top: 12vh;
	}

	.palette {
		width: min(640px, calc(100vw - 32px));
		background: var(--bg);
		border: 1px solid var(--hairline-strong);
		box-shadow: 0 8px 32px rgba(0, 0, 0, 0.12);
	}

	.palette input {
		border: none;
		border-bottom: 1px solid var(--hairline);
		background: var(--bg);
		padding: 12px 14px;
		font-size: 14px;
	}

	.results {
		list-style: none;
		margin: 0;
		padding: 4px 0;
		max-height: 50vh;
		overflow-y: auto;
	}

	.results li {
		padding: 0;
	}

	.result {
		display: grid;
		grid-template-columns: minmax(120px, auto) 1fr auto;
		gap: 12px;
		align-items: baseline;
		width: 100%;
		text-align: left;
		border: none;
		background: none;
		padding: 6px 14px;
	}

	.result.selected {
		background: var(--code-bg);
	}

	.result-name {
		font-weight: 600;
		white-space: nowrap;
	}

	.result-name .hit {
		color: var(--accent);
	}

	.result-alias {
		font-weight: 400;
		color: var(--faint);
		margin-left: 6px;
	}

	.result-desc {
		font-family: var(--font-sans);
		color: var(--muted);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.result-where {
		white-space: nowrap;
	}

	.empty {
		padding: 12px 14px;
	}

	@media (max-width: 768px) {
		.palette-overlay {
			padding-top: 0;
		}

		.palette {
			width: 100vw;
			border-left: none;
			border-right: none;
		}

		.result-desc {
			display: none;
		}
	}
</style>
