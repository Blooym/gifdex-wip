<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { HTMLButtonAttributes } from 'svelte/elements';

	type Variant = 'neutral' | 'destructive' | 'primary';
	type Size = 'small' | 'normal' | 'large';

	interface Props extends HTMLButtonAttributes {
		variant?: Variant;
		size?: Size;
		class?: string;
		children: Snippet;
	}

	let {
		variant = 'primary',
		size = 'normal',
		class: className,
		children,
		...restProps
	}: Props = $props();
</script>

<button class="button {variant} {size} {className}" {...restProps}>
	{@render children()}
</button>

<style>
	.button {
		border: none;
		border-radius: 6px;
		font-weight: 600;
		cursor: pointer;
		transition: all 0.2s;
		flex-shrink: 0;
	}

	.button:disabled {
		cursor: not-allowed;
		opacity: 0.5;
	}

	/* Sizes */
	.small {
		padding: 6px 12px;
		font-size: 0.7rem;
	}

	.normal {
		padding: 10px 20px;
		font-size: 0.8rem;
	}

	.large {
		padding: 14px 28px;
		font-size: 1rem;
	}

	/* Variants */
	.primary {
		background: var(--ctp-mauve);
		color: var(--ctp-crust);
	}

	.primary:hover:not(:disabled) {
		background: var(--ctp-lavender);
	}

	.neutral {
		background: transparent;
		color: var(--ctp-text);
		border: 2px solid var(--ctp-surface0);
	}

	.neutral:hover:not(:disabled) {
		border-color: var(--ctp-mauve);
		color: var(--ctp-mauve);
	}

	.destructive {
		background: var(--ctp-red);
		color: var(--ctp-crust);
	}

	.destructive:hover:not(:disabled) {
		background: var(--ctp-maroon);
	}
</style>
