<script lang="ts">
	import { getErrors, dismiss } from '$lib/errors.svelte';

	let errors = $derived(getErrors());
</script>

{#if errors.length > 0}
	<div class="error-bar">
		{#each errors as error (error.id)}
			<div class="error-item">
				<span>{error.message}</span>
				<button onclick={() => dismiss(error.id)}>×</button>
			</div>
		{/each}
	</div>
{/if}
