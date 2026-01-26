<script lang="ts">
	import { onMount, type Snippet } from 'svelte';
	import { getDropdownMenuContext } from './DropdownMenu.svelte';

	const { uniqueIdentifier, registerContentMenuRef: registerContent } = getDropdownMenuContext();

	const { children }: { children: Snippet } = $props();

	let menuRef: HTMLMenuElement;

	onMount(() => {
		registerContent(menuRef);
	});
</script>

<menu
	bind:this={menuRef}
	id={uniqueIdentifier}
	popover
	class="dropdown-content"
	style="position-anchor: --{uniqueIdentifier}"
>
	{@render children()}
</menu>

<style>
	.dropdown-content {
		position: absolute;
		position-area: center bottom;
		width: max-content;
		min-width: 12.5rem;

		background: var(--ctp-mantle);
		border: var(--border-sm) solid var(--ctp-surface0);
		border-radius: var(--radius-lg);

		margin-block-start: 8px;
		position-try-fallbacks:
			block-end span-inline-start,
			block-end span-inline-end,
			--top,
			--top-span-start,
			--top-span-end;
	}

	@position-try --top {
		position-area: block-start center;
		margin-block-start: 0;
		margin-block-end: 8px;
	}

	@position-try --top-span-start {
		position-area: block-start span-inline-start;
		margin-block-start: 0;
		margin-block-end: 8px;
	}

	@position-try --top-span-end {
		position-area: block-start span-inline-end;
		margin-block-start: 0;
		margin-block-end: 8px;
	}
</style>
