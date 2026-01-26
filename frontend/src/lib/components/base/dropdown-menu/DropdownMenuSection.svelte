<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		children: Snippet;
		label?: string;
	}

	let { children, label }: Props = $props();
</script>

<section class="dropdown-section">
	{#if label}
		<span class="section-label">{label}</span>
	{/if}
	{@render children()}
</section>

<style>
	.dropdown-section {
		display: flex;
		flex-direction: column;
		padding: 4px;
	}

	.section-label {
		padding: 4px 8px;
		font-size: 0.75rem;
		font-weight: 600;
		color: var(--ctp-subtext0);
		text-transform: uppercase;
	}
</style>
