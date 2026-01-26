import { NetGifdexActorGetProfile } from '$lib/lexicons';
import { authStore } from '$lib/stores/auth.svelte';
import type { Did } from '@atcute/lexicons';
import { isDid } from '@atcute/lexicons/syntax';
import { error } from '@sveltejs/kit';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ params }) => {
	const { actor } = params;

	if (!isDid(actor)) {
		return error(400, {
			message: 'Invalid profile identifier, please double check you have entered it correctly.'
		});
	}

	// TODO: Handling account deactivated correctly.
	let profile;
	try {
		profile = await authStore.client.call(NetGifdexActorGetProfile, {
			params: {
				actor: actor as Did
			}
		});
	} catch (err: unknown) {
		if (err instanceof Error) {
			return error(500, {
				message: err.message
			});
		}
		return error(500, { message: 'An unknown error occured.' });
	}
	if (!profile.ok) {
		if (profile.status === 404) {
			return error(404, {
				message: 'This profile does not exist or has been deleted.'
			});
		}
		return error(profile.status, {
			message: `${profile.data.error}${profile.data.message ? `: ${profile.data.message}` : ''}`
		});
	}

	return {
		actor,
		profile: profile.data
	};
};
