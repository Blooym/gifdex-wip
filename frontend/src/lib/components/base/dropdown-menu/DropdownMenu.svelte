<script lang="ts" module>
	import { createContext } from 'svelte';

	interface DropdownMenuContext {
		/**
		 * The component-unique identifier used for target and anchoring.
		 */
		uniqueIdentifier: string;

		/**
		 * Callback to manually close this dropdown element outside of its regular integrated handling.
		 */
		close: () => void;

		/**
		 * Register the associated content menu of this dropdown.
		 */
		registerContentMenuRef: (element: HTMLMenuElement) => void;
	}

	const [getDropdownMenuContext, setDropdownMenuContext] = createContext<DropdownMenuContext>();

	export { getDropdownMenuContext };
</script>

<script lang="ts">
	import type { Snippet } from 'svelte';

	const { children }: { children: Snippet } = $props();

	// Svelte doesn't scope anchors or popover targets.
	// This identifier is used to scope them by
	// dropdown tree for popover and anchoring.
	const identifier = crypto.randomUUID();

	let menuContentRef: HTMLMenuElement;

	setDropdownMenuContext({
		uniqueIdentifier: identifier,
		close: () => menuContentRef.hidePopover(),
		registerContentMenuRef: (el) => (menuContentRef = el)
	});
</script>

<div>
	{@render children()}
</div>

<style>
	div {
		display: contents;
	}
</style>
