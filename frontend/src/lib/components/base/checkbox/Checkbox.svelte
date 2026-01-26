<script lang="ts">
	import type { HTMLInputAttributes } from 'svelte/elements';

	interface Props extends Omit<HTMLInputAttributes, 'checked'> {
		label: string;
		hint?: string;
		checked?: boolean;
		surface?: 'base' | 'mantle';
	}

	let {
		label,
		hint,
		checked = $bindable(false),
		surface = 'base',
		disabled,
		...restProps
	}: Props = $props();
</script>

<label class="checkbox surface-{surface}" class:disabled>
	<input type="checkbox" bind:checked {disabled} {...restProps} />
	<span class="checkbox-label">{label}</span>
	{#if hint}
		<span class="checkbox-hint">{hint}</span>
	{/if}
</label>

<style>
	.checkbox {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 10px 14px;
		border-radius: var(--radius-md);
		cursor: pointer;
		transition: all 0.2s;

		&.disabled {
			cursor: not-allowed;
			opacity: 0.5;
		}

		&:hover:not(.disabled) {
			border-color: var(--ctp-surface1);
		}

		input[type='checkbox'] {
			width: 16px;
			height: 16px;
			accent-color: var(--ctp-mauve);
			cursor: pointer;
			&:disabled {
				cursor: not-allowed;
			}
		}

		.checkbox-label {
			color: var(--ctp-text);
			font-size: 0.9rem;
			font-weight: 500;
		}

		.checkbox-hint {
			color: var(--ctp-subtext0);
			font-weight: 400;
			font-size: 0.75rem;
			margin-left: auto;
		}
	}

	.checkbox.surface-base {
		background: var(--ctp-mantle);
		border: var(--border-sm) solid var(--ctp-surface0);
	}

	.checkbox.surface-mantle {
		background: var(--ctp-crust);
		border: var(--border-sm) solid var(--ctp-surface0);
	}
</style>
