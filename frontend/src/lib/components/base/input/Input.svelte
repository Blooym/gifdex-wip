<script lang="ts">
	import type { HTMLInputAttributes } from 'svelte/elements';

	type Surface = 'base' | 'mantle';

	interface Props extends Omit<HTMLInputAttributes, 'size' | 'value'> {
		surface?: Surface;
		size?: 'small' | 'normal';
		value?: string;
		class?: string;
	}

	let {
		surface = 'base',
		size = 'normal',
		value = $bindable(''),
		class: className = '',
		...restProps
	}: Props = $props();
</script>

<input class="{surface} {size} {className}" bind:value {...restProps} />

<style>
	input {
		width: 100%;
		border-radius: var(--radius-md);
		color: var(--ctp-text);
		font-family: inherit;
		transition: border-color 0.2s;
	}

	input:focus {
		outline: none;
		border-color: var(--ctp-mauve);
	}

	/* Surface variants */
	input.base {
		background: var(--ctp-mantle);
		border: var(--border-md) solid var(--ctp-surface0);
	}

	input.mantle {
		background: var(--ctp-crust);
		border: var(--border-md) solid var(--ctp-surface1);
	}

	input::placeholder {
		color: var(--ctp-subtext0);
	}

	/* Sizes */
	.small {
		padding: 6px 12px;
		font-size: 0.75rem;
	}

	.normal {
		padding: 10px 16px;
		font-size: 0.875rem;
	}

	.large {
		padding: 14px 20px;
		font-size: 1rem;
	}
</style>
