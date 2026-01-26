import { NetGifdexFeedGetPost } from '$lib/lexicons';
import { authStore } from '$lib/stores/auth.svelte';
import type { Did } from '@atcute/lexicons';
import { isCid, isDid, isTid } from '@atcute/lexicons/syntax';
import { error } from '@sveltejs/kit';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ params }) => {
	const { actor, rkey } = params;

	const [tid, cid] = rkey.split(':', 2);
	if (!isDid(actor)) {
		return error(400, {
			message: 'Invalid profile identifier, please double check you have entered it correctly.'
		});
	}
	if (!isTid(tid) || !isCid(cid)) {
		return error(400, {
			message: 'Malformed post identifier, please double check you have entered it correctly.'
		});
	}

	// TODO: Handling account deactivated correctly.
	let post;
	try {
		post = await authStore.client.call(NetGifdexFeedGetPost, {
			params: {
				actor: actor as Did,
				rkey
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
	if (!post.ok) {
		if (post.status === 404) {
			return error(404, {
				message: 'This post does not exist or has been deleted.'
			});
		}
		return error(post.status, {
			message: `${post.data.error}${post.data.message ? `: ${post.data.message}` : ''}`
		});
	}

	return {
		post: post.data.post
	};
};
