<script lang="ts">
	import { XIcon } from 'lucide-svelte';
	import type { Snippet } from 'svelte';
	import Button from '../button/Button.svelte';

	interface Props {
		open?: boolean;
		allowClose?: boolean;
		children: Snippet;
	}

	let { open = $bindable(false), children, allowClose = true }: Props = $props();
	let dialog = $state<HTMLDialogElement | undefined>();

	$effect(() => {
		if (open) dialog?.showModal();
		if (!open) dialog?.close();
	});

	function handleBackdropClick(e: MouseEvent) {
		if (allowClose && e.target === dialog) {
			dialog.close();
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if (!allowClose && e.key === 'Escape') {
			e.preventDefault();
		}
	}
</script>

<dialog
	bind:this={dialog}
	onclose={() => (open = false)}
	onclick={handleBackdropClick}
	onkeydown={handleKeydown}
>
	{#if allowClose}
		<div class="close-button-container">
			<Button
				variant="neutral"
				surface="mantle"
				size="small"
				onclick={() => dialog?.close()}
				title="Close dialog"
				aria-label="Close dialog"
			>
				<XIcon size={18} />
			</Button>
		</div>
	{/if}
	{@render children()}
</dialog>

<style>
	dialog {
		position: relative;
		width: 95%;
		max-width: 36rem;
		max-height: 95%;
		border-radius: var(--radius-lg);
		background: var(--ctp-mantle);
		color: var(--ctp-text);
		border: var(--border-sm) solid var(--ctp-surface0);
		padding: 24px 12px;

		&::backdrop {
			background: rgba(0, 0, 0, 0.6);
		}

		@media (prefers-reduced-motion: no-preference) {
			&[open] {
				animation: zoom 0.1s cubic-bezier(0.34, 1.56, 0.64, 1);

				&::backdrop {
					animation: fade 0.1s ease-out;
				}
			}
		}
	}

	.close-button-container {
		position: absolute;
		top: 8px;
		right: 8px;
	}

	@media (prefers-reduced-motion: no-preference) {
		@keyframes zoom {
			from {
				transform: scale(0.95);
			}
			to {
				transform: scale(1);
			}
		}

		@keyframes fade {
			from {
				opacity: 0;
			}
			to {
				opacity: 1;
			}
		}
	}
</style>
