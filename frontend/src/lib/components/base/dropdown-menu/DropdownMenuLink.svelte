<script lang="ts">
	import type { Snippet } from 'svelte';
	import { getDropdownMenuContext } from './DropdownMenu.svelte';

	interface Props {
		children: Snippet;
		href: string;
		variant?: 'default' | 'danger';
		disabled?: boolean;
		autoClose?: boolean;
		grow?: boolean;
	}

	let { children, href, variant = 'default', disabled = false, autoClose = true, grow = true }: Props = $props();

	const dropdown = getDropdownMenuContext();

	function handleClick(event: MouseEvent) {
		if (disabled) {
			event.preventDefault();
			return;
		}
		if (autoClose) dropdown?.close();
	}
</script>

<a
	href={disabled ? undefined : href}
	class="dropdown-item"
	class:danger={variant === 'danger'}
	class:disabled
	class:grow
	aria-disabled={disabled}
	onclick={handleClick}
>
	{@render children()}
</a>

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
		text-decoration: none;
		text-align: left;
		cursor: pointer;
		transition: background 150ms;
	}

	.dropdown-item.grow {
		flex: 1 1 0%;
	}

	.dropdown-item:hover:not(.disabled) {
		background: var(--ctp-surface0);
	}

	.dropdown-item.danger {
		color: var(--ctp-red);
	}

	.dropdown-item.danger:hover:not(.disabled) {
		background: color-mix(in srgb, var(--ctp-red) 15%, transparent);
	}

	.dropdown-item.disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}
</style>
