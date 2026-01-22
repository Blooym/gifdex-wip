import { PUBLIC_APPVIEW_DID } from '$env/static/public';
import { NetGifdexActorGetProfile } from '$lib/lexicons';
import type { ProfileView } from '$lib/lexicons/types/net/gifdex/actor/defs';
import { Client, ok } from '@atcute/client';
import type { Did } from '@atcute/lexicons';
import type { OAuthUserAgent, Session } from '@atcute/oauth-browser-client';

const APPVIEW_SERVICE_ID = '#gifdex_appview';

export class User {
	readonly did: Did;

	profile = $state<ProfileView | null>(null);
	isLoadingProfile = $state(false);
	profileError = $state<string | null>(null);

	private session: Session;
	private agent: OAuthUserAgent;
	readonly client: Client;

	constructor(did: Did, session: Session, agent: OAuthUserAgent) {
		this.did = did;
		this.session = session;
		this.agent = agent;
		this.client = new Client({
			handler: agent,
			proxy: {
				did: PUBLIC_APPVIEW_DID as Did,
				serviceId: APPVIEW_SERVICE_ID
			}
		});
		this.fetchProfile();
	}

	/**
	 * Fetch fresh profile data from server and update cache
	 */
	async fetchProfile(): Promise<void> {
		if (this.isLoadingProfile) return;

		this.isLoadingProfile = true;
		this.profileError = null;

		try {
			this.profile = await ok(
				this.client.call(NetGifdexActorGetProfile, {
					params: {
						actor: this.did
					}
				})
			);
		} catch (err) {
			console.error('Failed to fetch profile:', err);
			this.profileError = 'Failed to load profile';
		} finally {
			this.isLoadingProfile = false;
		}
	}

	getAgent(): OAuthUserAgent {
		return this.agent;
	}

	getSession(): Session {
		return this.session;
	}
}
