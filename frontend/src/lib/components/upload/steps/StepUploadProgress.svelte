<script lang="ts">
	import Button from '$lib/components/base/button/Button.svelte';
	import { CircleAlert } from 'lucide-svelte';
	import { uploadState } from '../state.svelte';

	const isError = $derived(uploadState.progress.status === 'error');

	function handleRetry() {
		uploadState.setProgress('uploading', 'Uploading your GIF...');
	}
</script>

<div class="step-progress">
	<div class="progress-indicator">
		{#if isError}
			<div class="icon-error">
				<CircleAlert size={48} />
			</div>
		{:else}
			<div class="spinner"></div>
		{/if}
	</div>

	<p class="progress-message">{uploadState.progress.message}</p>

	{#if isError}
		<div class="step-actions">
			<Button variant="neutral" onclick={handleRetry}>Try Again</Button>
		</div>
	{/if}
</div>

<style>
	.step-progress {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 24px;
		padding: 40px 20px;
		min-height: 250px;
	}

	.progress-indicator {
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.spinner {
		width: 48px;
		height: 48px;
		border: 4px solid var(--ctp-surface1);
		border-top-color: var(--ctp-mauve);
		border-radius: 50%;
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}

	.icon-error {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 72px;
		height: 72px;
		border-radius: 50%;
		background: var(--ctp-red);
		color: var(--ctp-crust);
	}

	.progress-message {
		margin: 0;
		font-size: 1rem;
		color: var(--ctp-text);
		text-align: center;
	}

	.step-actions {
		display: flex;
		justify-content: center;
		gap: 12px;
		margin-top: 8px;
	}
</style>
