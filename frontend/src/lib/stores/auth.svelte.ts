// src/lib/auth.svelte.ts
import { invalidateAll } from '$app/navigation';
import {
	PUBLIC_APPVIEW_DID,
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
	getSession,
	OAuthUserAgent,
	type Session
} from '@atcute/oauth-browser-client';

const LAST_SIGNED_IN_LOCALKEY = 'lastSignedIn';
const OAUTH_REDIRECT_KEY = 'oauth-session-storage';
const APPVIEW_SERVICE_ID = '#gifdex_appview';

enum OAuthAuthenticationType {
	Account,
	PDS
}

class AuthStore {
	client = $state<Client>(this.createUnauthenticatedClient());
	oauthAgent = $state<OAuthUserAgent | null>(null);
	session = $state<Session | null>(null);
	promptSignIn = $state(false);

	/**
	 * Initialises required dependencies for performing OAuth operations.
	 */
	private setupOAuth() {
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
	 * Initialise the auth store.
	 *
	 * This method will setup OAuth ahead of time and make an attempt to restore the last
	 * in-use session if available, falling back to an unauthenticated session otherwise.
	 */
	async initialize() {
		this.setupOAuth();
		try {
			const restoredSession = await this.restoreSession();
			if (restoredSession) {
				const agent = new OAuthUserAgent(restoredSession);
				this.session = restoredSession;
				this.oauthAgent = agent;
				this.client = this.createAuthenticatedClient(agent);
			} else {
				this.client = this.createUnauthenticatedClient();
			}
		} catch (err) {
			console.error('Failed to restore session:', err);
			this.client = this.createUnauthenticatedClient();
		}
	}

	/**
	 * Create a new session from the current URL fragment and immediately switch to it if successful.
	 *
	 * @returns Whether creating and switching to the new session was successful.
	 */
	async createSession(): Promise<{ success: boolean; redirect: string | null }> {
		const params = new URLSearchParams(location.hash.slice(1));
		if (!params.has('state') || (!params.has('code') && !params.has('error'))) {
			return { success: false, redirect: null };
		}

		history.replaceState(null, '', location.pathname + location.search);

		try {
			const auth = await finalizeAuthorization(params);
			const did = auth.session.info.sub;
			localStorage.setItem(LAST_SIGNED_IN_LOCALKEY, did);

			const agent = new OAuthUserAgent(auth.session);
			this.session = auth.session;
			this.oauthAgent = agent;
			this.client = this.createAuthenticatedClient(agent);
			const redirect = sessionStorage.getItem(OAUTH_REDIRECT_KEY);
			sessionStorage.removeItem(OAUTH_REDIRECT_KEY);
			return { success: true, redirect };
		} catch (err) {
			console.error('Failed to create session:', err);
			return { success: false, redirect: null };
		}
	}

	/**
	 * Restore the last session that was used.
	 */
	private async restoreSession(): Promise<Session | null> {
		const lastSignedIn = localStorage.getItem(LAST_SIGNED_IN_LOCALKEY);
		if (!lastSignedIn) {
			return null;
		}

		try {
			return await getSession(lastSignedIn as Did, { allowStale: true });
		} catch (err) {
			deleteStoredSession(lastSignedIn as Did);
			localStorage.removeItem(LAST_SIGNED_IN_LOCALKEY);
			throw err;
		}
	}

	/**
	 * Whether the auth store is currently authenticated with a valid session or not.
	 */
	isAuthenticated(): this is this & {
		session: Session;
		oauthAgent: OAuthUserAgent;
	} {
		return this.session != null && this.oauthAgent != null;
	}

	/**
	 * Initiate an OAuth sign in flow for the provided identifier.
	 *
	 * @param identifier Either a Handle, DID or PDS url.
	 * @returns Whether initiating the OAuth sign-in was successful.
	 */
	async oauthSignIn(identifier: string): Promise<boolean> {
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
			// Strip leading @ if present as OAuth doesn't accept this
			const cleanIdentifier = identifier.startsWith('@') ? identifier.substring(1) : identifier;
			const authUrl = await createAuthorizationUrl({
				target:
					authType === OAuthAuthenticationType.Account
						? { type: 'account', identifier: cleanIdentifier as ActorIdentifier }
						: { type: 'pds', serviceUrl: cleanIdentifier },
				scope: PUBLIC_OAUTH_SCOPE
			});
			sessionStorage.setItem(OAUTH_REDIRECT_KEY, window.location.toString()); // For automatic redirection back to the current page.
			window.location.assign(authUrl);
			return true;
		} catch (err) {
			console.error('Failed to create authorization URL:', err);
			return false;
		}
	}

	/**
	 * Revoke the current session token and refresh the current page.
	 */
	async oauthSignOut(): Promise<void> {
		if (this.session && this.oauthAgent) {
			await this.oauthAgent.signOut();
			deleteStoredSession(this.session.info.sub as Did);
		} else {
			console.warn('Attempted to sign out without an active session');
			return;
		}
		localStorage.removeItem(LAST_SIGNED_IN_LOCALKEY);
		this.session = null;
		this.oauthAgent = null;
		this.client = this.createUnauthenticatedClient();
		sessionStorage.removeItem(OAUTH_REDIRECT_KEY);
		await invalidateAll();
		window.location.reload();
	}

	/**
	 * Create a new authenticated client using the provided OAuth agent and configure it
	 * to proxy through the default AppView service.
	 */
	private createAuthenticatedClient(agent: OAuthUserAgent): Client {
		return new Client({
			handler: agent,
			proxy: {
				did: PUBLIC_APPVIEW_DID as Did,
				serviceId: APPVIEW_SERVICE_ID
			}
		});
	}

	/**
	 * Create a new unauthenticated client using the default AppView URL.
	 */
	private createUnauthenticatedClient(): Client {
		return new Client({
			handler: simpleFetchHandler({
				service: PUBLIC_APPVIEW_URL
			})
		});
	}

	/**
	 * Determine whether the given identifier is valid/supported for authentication operations.
	 * @param identifier The identifier to analyse
	 * @returns True if supported, false if unsupported or type is unknown.
	 */
	isValidIdentifier(identifier: string): boolean {
		return this.getAuthTypeForIdentifier(identifier) !== null;
	}

	/**
	 * Determine the type of authentication required for the given identifier.
	 * @param identifier The identifier to analyse.
	 * @returns Either the `OAuthAuthenticationType` if determined, or null if the identifier is invalid or could not be determined.
	 */
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
				// Fixed: was 'http://'
				return OAuthAuthenticationType.PDS;
			}
		} catch {
			//
		}

		return null;
	}
}

export const authStore = new AuthStore();
