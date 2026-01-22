// src/hooks.client.ts
import { authStore } from '$lib/stores/auth.svelte';
import type { ClientInit } from '@sveltejs/kit';

export const init: ClientInit = async () => {
	await authStore.initialize();
};
