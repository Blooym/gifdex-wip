<script lang="ts">
	import { page } from '$app/state';
	import { CircleAlert, House, RotateCw, SearchX } from 'lucide-svelte';
	const isClientError = $derived(page.status >= 400 && page.status < 500);
	const isNotFound = $derived(page.status === 404);
	const Icon = $derived(isNotFound ? SearchX : CircleAlert);
</script>

<div class="error-page">
	<Icon size={48} class="error-icon" />
	<h1>{page.status}</h1>
	<p>{page.error?.message || (isNotFound ? 'Page not found' : 'Something went wrong')}</p>
	<div class="actions">
		{#if !isClientError}
			<button class="action-button" onclick={() => location.reload()}>
				<RotateCw size={18} />
				Reload
			</button>
		{/if}
		<a href="/" class="action-button">
			<House size={18} />
			Homepage
		</a>
	</div>
</div>

<style>
	.error-page {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		min-height: 60vh;
		padding: 40px;
		color: var(--ctp-text);
		gap: 1rem;
	}

	.error-page :global(.error-icon) {
		color: var(--ctp-subtext0);
	}

	h1 {
		font-size: 3rem;
		color: var(--ctp-text);
		margin: 0;
	}

	p {
		font-size: 1rem;
		color: var(--ctp-subtext0);
		margin: 0;
	}

	.actions {
		display: flex;
		gap: 0.75rem;
		margin-top: 1rem;
	}

	.action-button {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		padding: 8px 16px;
		background: transparent;
		color: var(--ctp-text);
		text-decoration: none;
		border: 2px solid var(--ctp-surface0);
		border-radius: 6px;
		font-weight: 600;
		font-size: 0.9rem;
		transition: all 0.2s;
		cursor: pointer;
		font-family: inherit;
	}

	.action-button:hover {
		border-color: var(--ctp-mauve);
		color: var(--ctp-mauve);
	}
</style>
