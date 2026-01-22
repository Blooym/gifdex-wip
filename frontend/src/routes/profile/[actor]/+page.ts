import { NetGifdexActorGetProfile } from '$lib/lexicons';
import { authStore } from '$lib/stores/auth.svelte';
import { ok } from '@atcute/client';
import type { Did } from '@atcute/lexicons';
import { isDid } from '@atcute/lexicons/syntax';
import { error } from '@sveltejs/kit';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ params }) => {
	const { actor } = params;

	if (!isDid(actor)) {
		return error(400, {
			message: 'Invalid DID'
		});
	}

	return {
		actor,
		profile: await ok(
			authStore.client.call(NetGifdexActorGetProfile, {
				params: {
					actor: actor as Did
				}
			})
		)
	};
};
