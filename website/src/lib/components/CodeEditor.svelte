<script lang="ts">
	import { untrack } from 'svelte';
	import { doux } from '$lib/doux';
	import { startScope, stopScope, registerActiveEditor, unregisterActiveEditor } from '$lib/scope';
	import { Play, Square } from 'lucide-svelte';

	interface Props {
		code?: string;
		rows?: number;
	}

	let { code = '', rows = 3 }: Props = $props();

	let textarea: HTMLTextAreaElement;
	let active = $state(false);
	let currentCode = $state(untrack(() => code));
	let evaluated = $state(false);

	const resetCallback = () => {
		active = false;
	};

	const highlight = (code: string) =>
		code
			.replace(/&/g, '&amp;')
			.replace(/</g, '&lt;')
			.replace(/>/g, '&gt;')
			.replace(/(\/\/.*)/g, '<span class="hl-comment">$1</span>')
			.replace(
				/(\/)([a-zA-Z_][a-zA-Z0-9_]*)/g,
				'<span class="hl-slash">$1</span><span class="hl-command">$2</span>'
			)
			.replace(
				/(\/)(-?[0-9]*\.?[0-9]+)/g,
				'<span class="hl-slash">$1</span><span class="hl-number">$2</span>'
			) + '\n';

	let highlighted = $derived(highlight(currentCode));

	function flash() {
		evaluated = false;
		setTimeout(() => {
			evaluated = true;
		}, 50);
	}

	async function run() {
		flash();
		await doux.ready;
		doux.evaluate({ doux: 'reset_schedule' });
		doux.evaluate({ doux: !active ? 'reset' : 'hush_endless' });

		const blocks = currentCode.split('\n\n').filter(Boolean);

		const msgs = await Promise.all(
			blocks.map((block) => {
				const event = doux.parsePath(block);
				return doux.prepare({ doux: 'play', ...event });
			})
		);

		if (!active) {
			doux.evaluate({ doux: 'reset_time' });
			registerActiveEditor(resetCallback);
			active = true;
			startScope();
		}

		msgs.forEach((msg) => doux.send(msg));
	}

	function stop() {
		active = false;
		unregisterActiveEditor(resetCallback);
		stopScope();
		doux.hush();
	}

	function handleKeydown(e: KeyboardEvent) {
		if ((e.ctrlKey || e.altKey) && e.key === 'Enter') {
			run();
		}
	}

	function handleScroll() {
		const pre = textarea.previousElementSibling as HTMLPreElement;
		if (pre) pre.scrollTop = textarea.scrollTop;
	}
</script>

<div class="repl">
	<div class="repl-editor">
		<pre class="hl-pre" aria-hidden="true">{@html highlighted}</pre>
		<textarea
			bind:this={textarea}
			bind:value={currentCode}
			spellcheck="false"
			{rows}
			class:evaluated
			onkeydown={handleKeydown}
			onscroll={handleScroll}
		></textarea>
	</div>
	<div class="repl-controls">
		{#if active}
			<button class="stop" onclick={stop}><Square size={16} /></button>
		{:else}
			<button class="play" onclick={run}><Play size={16} /></button>
		{/if}
	</div>
</div>
