import { PUBLIC_APPVIEW_DID } from '$env/static/public';
import { NetGifdexActorGetProfile } from '$lib/lexicons';
import type { ProfileView } from '$lib/lexicons/types/net/gifdex/actor/defs';
import type { Main as Profile } from '$lib/lexicons/types/net/gifdex/actor/profile';
import { ComAtprotoRepoGetRecord, ComAtprotoRepoPutRecord } from '@atcute/atproto';
import { Client, ok } from '@atcute/client';
import type { Did } from '@atcute/lexicons';
import { OAuthUserAgent, type Session } from '@atcute/oauth-browser-client';
import { SvelteDate } from 'svelte/reactivity';

const APPVIEW_SERVICE_ID = '#gifdex_appview';

export class User {
	readonly did: Did;

	private _profile = $state<ProfileView | null>(null);
	private _session: Session;
	private _agent: OAuthUserAgent;
	private _profileError = $state<Error | null>(null);
	private _isLoadingProfile = $state(false);

	readonly client: Client;

	constructor(did: Did, session: Session) {
		this.did = did;
		this._session = session;
		this._agent = new OAuthUserAgent(session);
		this.client = new Client({
			handler: this._agent,
			proxy: {
				did: PUBLIC_APPVIEW_DID as Did,
				serviceId: APPVIEW_SERVICE_ID
			}
		});

		this.refreshProfile();
	}

	get session(): Session {
		return this._session;
	}

	get agent(): OAuthUserAgent {
		return this._agent;
	}

	get profile(): ProfileView | null {
		return this._profile;
	}

	get profileLoadError(): Error | null {
		return this._profileError;
	}

	isLoadingProfile(): boolean {
		return this._isLoadingProfile;
	}

	/**
	 * Fetch fresh profile data from server and update cache
	 */
	async refreshProfile(): Promise<void> {
		if (this._isLoadingProfile) return;

		this._isLoadingProfile = true;
		this._profileError = null;

		try {
			const profile = await ok(
				this.client.call(NetGifdexActorGetProfile, {
					params: {
						actor: this.did
					}
				})
			);
			this._profile = profile;
		} catch (err) {
			console.error('Failed to fetch profile:', err);
			this._profileError = err instanceof Error ? err : new Error('Failed to load profile');
			this._isLoadingProfile = false;
		} finally {
			this._isLoadingProfile = false;
		}
	}

	/**
	 * Update profile on server and refresh local state
	 *
	 * @param changes Partial profile changes to apply
	 * @returns true on success, false on failure
	 */
	async updateProfile(
		changes: Partial<Pick<Profile, 'displayName' | 'pronouns' | 'avatar'>>
	): Promise<boolean> {
		try {
			// Get current profile record from PDS
			let currentRecord: Profile | null = null;
			try {
				const response = await ok(
					this.client.call(ComAtprotoRepoGetRecord, {
						params: {
							repo: this.did,
							collection: 'net.gifdex.actor.profile',
							rkey: 'self'
						}
					})
				);
				currentRecord = response.value as Profile;
			} catch {
				// No existing profile record, create new one
				currentRecord = null;
			}

			// Merge changes with existing data
			const updatedRecord: Profile = {
				$type: 'net.gifdex.actor.profile',
				createdAt: currentRecord?.createdAt ?? new SvelteDate().toISOString(),
				displayName: changes.displayName ?? currentRecord?.displayName,
				pronouns: changes.pronouns ?? currentRecord?.pronouns,
				avatar: changes.avatar ?? currentRecord?.avatar
			};

			// Write to PDS
			await ok(
				this.client.call(ComAtprotoRepoPutRecord, {
					input: {
						repo: this.did,
						collection: 'net.gifdex.actor.profile',
						rkey: 'self',
						record: updatedRecord
					}
				})
			);

			// Refresh profile from AppView to get processed data (avatar URL, etc.)
			await this.refreshProfile();

			return true;
		} catch (err) {
			console.error('Failed to update profile:', err);
			return false;
		}
	}
}
