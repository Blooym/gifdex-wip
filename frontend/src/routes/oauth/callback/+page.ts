import { authStore } from '$lib/stores/auth.svelte';
import { error, redirect } from '@sveltejs/kit';
import type { PageLoad } from './$types';

export const load: PageLoad = async () => {
	const createSession = await authStore.createSession();

	if (!createSession.success) {
		return error(400, 'Failed to create session from callback URL');
	}

	return redirect(303, createSession.redirect ?? '/');
};
