<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { HTMLAnchorAttributes } from 'svelte/elements';

	interface Props extends HTMLAnchorAttributes {
		class?: string;
		children: Snippet;
	}

	let { class: className, children, ...restProps }: Props = $props();
</script>

<a class={className} {...restProps}>
	{@render children()}
</a>

<style>
	a {
		color: var(--ctp-mauve);
		text-decoration: none;
	}

	a:hover {
		text-decoration: underline;
	}
</style>
