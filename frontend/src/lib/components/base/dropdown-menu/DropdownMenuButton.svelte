<script lang="ts">
	import type { Snippet } from 'svelte';
	import { getDropdownMenuContext } from './DropdownMenu.svelte';

	interface Props {
		children: Snippet;
		onclick: (event: MouseEvent) => void;
		variant?: 'default' | 'danger';
		disabled?: boolean;
		autoClose?: boolean;
		grow?: boolean;
	}

	let {
		children,
		onclick,
		variant = 'default',
		disabled = false,
		autoClose = true,
		grow = true
	}: Props = $props();

	const dropdown = getDropdownMenuContext();

	function handleClick(event: MouseEvent) {
		if (disabled) return;
		onclick(event);
		if (autoClose) dropdown?.close();
	}
</script>

<button
	type="button"
	class="dropdown-item"
	class:danger={variant === 'danger'}
	class:grow
	{disabled}
	onclick={handleClick}
>
	{@render children()}
</button>

<style>
	.dropdown-item {
		display: flex;
		align-items: center;
		gap: 8px;
		min-width: 0;
		padding: 8px;
		border: none;
		border-radius: var(--radius-sm);
		background: transparent;
		color: var(--ctp-text);
		font-size: 0.875rem;
		font-family: inherit;
		text-align: left;
		cursor: pointer;
		transition: background 150ms;
	}

	.dropdown-item.grow {
		flex: 1 1 0%;
	}

	.dropdown-item:hover:not(:disabled) {
		background: var(--ctp-surface0);
	}

	.dropdown-item:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.dropdown-item.danger {
		color: var(--ctp-red);
	}

	.dropdown-item.danger:hover:not(:disabled) {
		background: color-mix(in srgb, var(--ctp-red) 15%, transparent);
	}
</style>
