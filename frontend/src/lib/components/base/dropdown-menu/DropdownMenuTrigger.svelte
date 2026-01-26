<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { HTMLButtonAttributes } from 'svelte/elements';
	import { getDropdownMenuContext } from './DropdownMenu.svelte';

	let {
		children,
		...props
	}: { children: Snippet } & Omit<HTMLButtonAttributes, 'popovertarget' | 'style'> = $props();

	const { uniqueIdentifier } = getDropdownMenuContext();
</script>

<button
	type="button"
	popovertarget={uniqueIdentifier}
	style="anchor-name: --{uniqueIdentifier}"
	{...props}
>
	{@render children()}
</button>

<style>
	button {
		display: inline-flex;
		align-items: center;
		gap: 8px;
		padding: 4px 8px;
		border: var(--border-md) solid var(--ctp-surface0);
		border-radius: var(--radius-md);
		background: vaR(--ctp-crust);
		color: var(--ctp-text);
		font-family: inherit;
		font-size: 0.875rem;
		cursor: pointer;
		transition: all 150ms;
	}

	button:hover {
		background: var(--ctp-surface0);
	}

	button:focus-visible {
		outline: 2px solid var(--ctp-blue);
		outline-offset: 2px;
	}
</style>
