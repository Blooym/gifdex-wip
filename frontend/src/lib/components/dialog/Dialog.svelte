<script lang="ts">
	import { XIcon } from 'lucide-svelte';
	import type { Snippet } from 'svelte';
	import Button from '../Button.svelte';

	interface Props {
		open?: boolean;
		showCloseButton?: boolean;
		children: Snippet;
	}

	let { open = $bindable(false), children, showCloseButton = true }: Props = $props();

	let dialog: HTMLDialogElement | undefined = $state();
	$effect(() => {
		if (open) dialog?.showModal();
		if (!open) dialog?.close();
	});
</script>

<dialog
	bind:this={dialog}
	onclose={() => (open = false)}
	onclick={(e) => {
		if (e.target === dialog) dialog.close();
	}}
>
	<div style="position: relative;">
		{#if showCloseButton}
			<Button
				variant="neutral"
				size="small"
				onclick={() => dialog?.close()}
				style="position: absolute; top: 0; right: 0;"
			>
				<XIcon size={16} />
			</Button>
		{/if}
		{@render children()}
	</div>
</dialog>

<style>
	dialog {
		width: 100%;
		max-width: 35em;
		border-radius: 12px;
		background: var(--ctp-mantle);
		color: var(--ctp-text);
		border: 1px solid var(--ctp-surface0);
		padding: 1.5rem;
	}

	dialog::backdrop {
		background: rgba(0, 0, 0, 0.7);
	}

	dialog[open] {
		animation: zoom 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
	}

	@keyframes zoom {
		from {
			transform: scale(0.95);
		}
		to {
			transform: scale(1);
		}
	}

	dialog[open]::backdrop {
		animation: fade 0.2s ease-out;
	}

	@keyframes fade {
		from {
			opacity: 0;
		}
		to {
			opacity: 1;
		}
	}
</style>
