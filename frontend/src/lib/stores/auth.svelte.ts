import { invalidateAll } from '$app/navigation';
import {
	PUBLIC_APPVIEW_URL,
	PUBLIC_OAUTH_CLIENT_ID,
	PUBLIC_OAUTH_REDIRECT_URI,
	PUBLIC_OAUTH_SCOPE
} from '$env/static/public';
import { Client, simpleFetchHandler } from '@atcute/client';
import {
	CompositeDidDocumentResolver,
	CompositeHandleResolver,
	DohJsonHandleResolver,
	LocalActorResolver,
	PlcDidDocumentResolver,
	WebDidDocumentResolver,
	WellKnownHandleResolver
} from '@atcute/identity-resolver';
import type { ActorIdentifier, Did } from '@atcute/lexicons';
import { isDid, isHandle } from '@atcute/lexicons/syntax';
import {
	configureOAuth,
	createAuthorizationUrl,
	deleteStoredSession,
	finalizeAuthorization,
	getSession
} from '@atcute/oauth-browser-client';
import { User } from './user.svelte';

const STORED_DIDS_KEY = 'gifdex:storedDids';
const ACTIVE_USER_KEY = 'gifdex:activeUser';
const OAUTH_REDIRECT_KEY = 'oauth-session-storage';

enum OAuthAuthenticationType {
	Account,
	PDS
}

class AuthStore {
	private _users = $state<Map<Did, User>>(new Map());
	private _activeUserDid = $state<Did | null>(null);
	private _unauthenticatedClient: Client;

	showSignInDialog = $state(false);

	constructor() {
		this._unauthenticatedClient = new Client({
			handler: simpleFetchHandler({
				service: PUBLIC_APPVIEW_URL
			})
		});
	}

	get client(): Client {
		return this.activeUser?.client ?? this._unauthenticatedClient;
	}

	get users(): readonly User[] {
		return [...this._users.values()];
	}

	get activeUser(): User | null {
		if (this._activeUserDid === null) return null;
		return this._users.get(this._activeUserDid) ?? null;
	}

	/**
	 * Type guard to check if user is authenticated.
	 * After this guard, activeUser is guaranteed to be non-null.
	 */
	isAuthenticated(): this is this & {
		activeUser: User;
	} {
		return this.activeUser !== null;
	}

	private setupOAuth(): void {
		configureOAuth({
			metadata: {
				client_id: PUBLIC_OAUTH_CLIENT_ID,
				redirect_uri: PUBLIC_OAUTH_REDIRECT_URI
			},
			storageName: 'gifdex-oauth',
			identityResolver: new LocalActorResolver({
				handleResolver: new CompositeHandleResolver({
					strategy: 'race',
					methods: {
						dns: new DohJsonHandleResolver({
							dohUrl: 'https://cloudflare-dns.com/dns-query?'
						}),
						http: new WellKnownHandleResolver()
					}
				}),
				didDocumentResolver: new CompositeDidDocumentResolver({
					methods: {
						plc: new PlcDidDocumentResolver(),
						web: new WebDidDocumentResolver()
					}
				})
			})
		});
	}

	/**
	 * Initialize the auth store.
	 *
	 * Sets up OAuth and attempts to restore all stored sessions.
	 */
	async initialize(): Promise<void> {
		this.setupOAuth();
		try {
			await this.restoreAllSessions();
		} catch (err) {
			console.error('Failed to restore sessions:', err);
		}
	}

	private async restoreAllSessions(): Promise<void> {
		const storedDids = this.getStoredDids();
		if (storedDids.length === 0) return;

		const restoredUsers: User[] = [];
		const failedDids: Did[] = [];

		for (const did of storedDids) {
			try {
				const session = await getSession(did, { allowStale: true });
				const user = new User(did, session);
				this._users.set(did, user);
				restoredUsers.push(user);
			} catch (err) {
				console.error(`Failed to restore session for ${did}:`, err);
				failedDids.push(did);
			}
		}

		this._users = new Map(this._users);

		for (const did of failedDids) {
			this.removeStoredDid(did);
			deleteStoredSession(did);
		}

		if (restoredUsers.length > 0) {
			const savedActiveUser = localStorage.getItem(ACTIVE_USER_KEY) as Did | null;
			if (savedActiveUser && this._users.has(savedActiveUser)) {
				this._activeUserDid = savedActiveUser;
			} else {
				this._activeUserDid = restoredUsers[0].did;
				localStorage.setItem(ACTIVE_USER_KEY, this._activeUserDid);
			}
		}
	}

	/**
	 * Finalize OAuth callback from URL fragment.
	 *
	 * @returns Object with success status and optional redirect URL
	 */
	async finalizeOAuthCallback(): Promise<{ success: boolean; redirect: string | null }> {
		const params = new URLSearchParams(location.hash.slice(1));
		if (!params.has('state') || (!params.has('code') && !params.has('error'))) {
			return { success: false, redirect: null };
		}

		history.replaceState(null, '', location.pathname + location.search);

		try {
			const auth = await finalizeAuthorization(params);
			const did = auth.session.info.sub;

			this.addStoredDid(did);
			this._users.set(did, new User(did, auth.session));
			this._users = new Map(this._users);
			this._activeUserDid = did;
			localStorage.setItem(ACTIVE_USER_KEY, did);

			const redirect = sessionStorage.getItem(OAUTH_REDIRECT_KEY);
			sessionStorage.removeItem(OAUTH_REDIRECT_KEY);
			return { success: true, redirect };
		} catch (err) {
			console.error('Failed to finalize OAuth:', err);
			return { success: false, redirect: null };
		}
	}

	/**
	 * Initiate OAuth sign in flow for the provided identifier.
	 *
	 * @param identifier Handle, DID, or PDS URL
	 * @returns Whether initiating the flow was successful
	 */
	async initiateOAuthFlow(identifier: string): Promise<boolean> {
		identifier = identifier.trim();
		if (!identifier) {
			console.error('Empty identifier provided');
			return false;
		}

		const authType = this.getAuthTypeForIdentifier(identifier);
		if (authType === null) {
			console.error('Invalid login identifier:', identifier);
			return false;
		}

		try {
			const cleanIdentifier = identifier.startsWith('@') ? identifier.substring(1) : identifier;
			const authUrl = await createAuthorizationUrl({
				target:
					authType === OAuthAuthenticationType.Account
						? { type: 'account', identifier: cleanIdentifier as ActorIdentifier }
						: { type: 'pds', serviceUrl: cleanIdentifier },
				scope: PUBLIC_OAUTH_SCOPE
			});
			sessionStorage.setItem(OAUTH_REDIRECT_KEY, window.location.toString());
			window.location.assign(authUrl);
			return true;
		} catch (err) {
			console.error('Failed to create authorization URL:', err);
			return false;
		}
	}

	/**
	 * Check if identifier is valid for authentication.
	 */
	isValidIdentifier(identifier: string): boolean {
		identifier = identifier.trim();
		return this.getAuthTypeForIdentifier(identifier) !== null;
	}

	private getAuthTypeForIdentifier(identifier: string): OAuthAuthenticationType | null {
		const cleanIdentifier = identifier.startsWith('@') ? identifier.substring(1) : identifier;

		// Account (Handle or DID)
		if (isHandle(cleanIdentifier) || isDid(cleanIdentifier)) {
			return OAuthAuthenticationType.Account;
		}

		// PDS (URL)
		try {
			const url = new URL(cleanIdentifier);
			if (url.protocol === 'https:' || url.protocol === 'http:') {
				return OAuthAuthenticationType.PDS;
			}
		} catch {
			// Not a valid URL
		}

		return null;
	}

	/**
	 * Switch to a different signed-in user.
	 * Reloads the page after switching.
	 */
	async switchUser(did: Did): Promise<void> {
		if (!this._users.has(did)) {
			console.error(`Cannot switch to unknown user: ${did}`);
			return;
		}
		this._activeUserDid = did;
		localStorage.setItem(ACTIVE_USER_KEY, did);
		invalidateAll();
		window.location.reload();
	}

	/**
	 * Sign out a specific user by DID.
	 * Reloads the page after signing out if it's the current user.
	 */
	async signOut(did: Did): Promise<void> {
		const user = this._users.get(did);
		if (!user) {
			console.warn(`Attempted to sign out unknown user: ${did}`);
			return;
		}

		await user.agent.signOut();
		deleteStoredSession(did);
		this._users.delete(did);
		this._users = new Map(this._users);
		this.removeStoredDid(did);

		// Update active user
		if (this._activeUserDid === did) {
			const remainingUsers = [...this._users.keys()];
			if (remainingUsers.length > 0) {
				this._activeUserDid = remainingUsers[0];
				localStorage.setItem(ACTIVE_USER_KEY, this._activeUserDid);
			} else {
				this._activeUserDid = null;
				localStorage.removeItem(ACTIVE_USER_KEY);
			}
			invalidateAll();
			window.location.reload();
		}
	}

	/**
	 * Sign out all users and clear all stored sessions.
	 * Reloads the page after signing out.
	 */
	async signOutAll(): Promise<void> {
		for (const user of this._users.values()) {
			await user.agent.signOut();
			deleteStoredSession(user.did);
		}

		this._users.clear();
		this._users = new Map(this._users);
		this._activeUserDid = null;
		localStorage.removeItem(STORED_DIDS_KEY);
		localStorage.removeItem(ACTIVE_USER_KEY);
		sessionStorage.removeItem(OAUTH_REDIRECT_KEY);

		window.location.reload();
	}

	private getStoredDids(): Did[] {
		try {
			const stored = localStorage.getItem(STORED_DIDS_KEY);
			if (!stored) return [];
			return JSON.parse(stored) as Did[];
		} catch {
			return [];
		}
	}

	private addStoredDid(did: Did): void {
		const dids = this.getStoredDids();
		if (!dids.includes(did)) {
			dids.push(did);
			localStorage.setItem(STORED_DIDS_KEY, JSON.stringify(dids));
		}
	}

	private removeStoredDid(did: Did): void {
		const dids = this.getStoredDids().filter((d) => d !== did);
		if (dids.length > 0) {
			localStorage.setItem(STORED_DIDS_KEY, JSON.stringify(dids));
		} else {
			localStorage.removeItem(STORED_DIDS_KEY);
		}
	}
}

export const authStore = new AuthStore();
