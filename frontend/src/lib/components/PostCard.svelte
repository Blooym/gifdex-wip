<script lang="ts">
	import { goto } from '$app/navigation';
	import Button from '$lib/components/base/button/Button.svelte';
	import Shimmer from '$lib/components/base/Shimmer.svelte';
	import type { PostFeedView } from '$lib/lexicons/types/net/gifdex/feed/defs';
	import { authStore } from '$lib/stores/auth.svelte';
	import { ComAtprotoRepoCreateRecord, ComAtprotoRepoDeleteRecord } from '@atcute/atproto';
	import type { ActorIdentifier } from '@atcute/lexicons';
	import { DownloadIcon, Share2Icon, Star } from 'lucide-svelte';

	const { post: initialPost }: { post: PostFeedView } = $props();
	let post = $derived(initialPost);
	const { did, rkey } = $derived(parseAtUri(post.uri));
	const aspectRatio = $derived(
		post.media?.dimensions?.height && post.media?.dimensions?.width
			? (post.media.dimensions.height / post.media.dimensions.width) * 100
			: 100
	);
	let imageLoaded = $state(false);

	function parseAtUri(uri: string) {
		const match = uri.match(/^at:\/\/([^/]+)\/([^/]+)\/(.+)$/);
		if (!match) {
			throw new Error(`Invalid AT URI: ${uri}`);
		}
		return {
			did: match[1],
			collection: match[2],
			rkey: match[3]
		};
	}

	function navigateToPost(event: MouseEvent) {
		event.stopPropagation();
		goto(`/profile/${did}/post/${rkey}`);
	}

	async function copyPostMediaUrl(event: MouseEvent) {
		event.stopPropagation();
		await navigator.clipboard.writeText(post.media.fullsizeUrl);
	}

	async function downloadPost(event: MouseEvent) {
		event.stopPropagation();
		throw new Error('Not implemented');
	}

	let updatingFavouriteStatus = $state(false);
	async function handleFavouriteChange(event: MouseEvent) {
		event.stopPropagation();

		if (!authStore.isAuthenticated()) {
			authStore.showSignInDialog = true;
			return;
		}

		if (updatingFavouriteStatus) {
			return;
		}

		updatingFavouriteStatus = true;
		try {
			switch (post.viewer.favourite === undefined) {
				case true: {
					const response = await authStore.client.call(ComAtprotoRepoCreateRecord, {
						input: {
							collection: 'net.gifdex.feed.favourite',
							record: {
								$type: 'net.gifdex.feed.favourite',
								subject: `at://${did}/net.gifdex.feed.post/${rkey}`,
								createdAt: new Date().toISOString()
							},
							repo: authStore.activeUser.did as ActorIdentifier
						}
					});
					if (response.ok) {
						post.favouriteCount += 1;
						post.viewer.favourite = parseAtUri(response.data.uri).rkey;
					}
					return;
				}
				case false: {
					const response = await authStore.client.call(ComAtprotoRepoDeleteRecord, {
						input: {
							collection: 'net.gifdex.feed.favourite',
							rkey: post.viewer.favourite!,
							repo: authStore.activeUser.did
						}
					});
					if (response.ok) {
						post.viewer.favourite = undefined;
						post.favouriteCount -= 1;
					}
					return;
				}
			}
		} finally {
			updatingFavouriteStatus = false;
		}
	}
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- TODO: Fix above -->
<div class="post-card" onclick={navigateToPost} role="button" tabindex="0">
	{#if post.media}
		<div class="media-container" style="padding-bottom: {aspectRatio}%;">
			{#if !imageLoaded}
				<Shimmer class="media-shimmer" radius={0} />
			{/if}
			<img
				src={post.media.thumbnailUrl}
				alt={post.media.alt || ''}
				class="post-media"
				class:loaded={imageLoaded}
				loading="lazy"
				width={post.media.dimensions?.width || ''}
				height={post.media.dimensions?.height || ''}
				onload={() => (imageLoaded = true)}
			/>
		</div>
	{/if}

	<div class="post-content">
		<h2 class="post-title">{post.title}</h2>
		<div class="post-author">
			{#if post.author.avatar}
				<img
					src={post.author.avatar}
					alt={post.author.displayName}
					loading="lazy"
					class="author-avatar"
				/>
			{/if}
			<div class="author-info">
				<div class="author-name">{post.author.displayName || 'Unknown'}</div>
				<div class="author-handle">@{post.author.handle || post.author.did}</div>
			</div>
			<div class="post-actions">
				<button
					class="favourite-button"
					class:favourited={post.viewer.favourite !== undefined}
					onclick={handleFavouriteChange}
					title={authStore.isAuthenticated()
						? post.viewer.favourite
							? 'Remove from favourites'
							: 'Add to favourites'
						: `${post.favouriteCount} favourites`}
				>
					<Star size={14} fill={post.viewer.favourite ? 'currentColor' : 'none'} />
					<span class="favourite-count"
						>{new Intl.NumberFormat('en', { notation: 'compact' }).format(
							post.favouriteCount
						)}</span
					>
				</button>
				{#if post.media}
					<Button title="Share" variant="neutral" size="small" onclick={copyPostMediaUrl}
						><Share2Icon size={14} /></Button
					>
					<Button title="Download" variant="neutral" size="small" onclick={downloadPost}
						><DownloadIcon size={14} /></Button
					>
				{/if}
			</div>
		</div>
	</div>
</div>

<style>
	.post-card {
		border-radius: var(--radius-md);
		overflow: hidden;
		transition: transform 0.2s;
		width: 100%;
		position: relative;
		cursor: pointer;
	}

	.post-card:hover,
	.post-card:focus-visible,
	.post-card:focus-within {
		transform: translateY(-2px);
	}

	.post-card:hover .post-content,
	.post-card:focus-visible .post-content,
	.post-card:focus-within .post-content {
		opacity: 1;
		pointer-events: auto;
	}

	.post-media {
		position: absolute;
		top: 0;
		left: 0;
		width: 100%;
		height: 100%;
		object-fit: contain;
		background: var(--ctp-crust);
		display: block;
		opacity: 0;
		transition: opacity 0.3s;
	}

	.post-media.loaded {
		opacity: 1;
	}

	.media-container {
		position: relative;
		width: 100%;
		background: var(--ctp-crust);
		overflow: hidden;
	}

	.media-container :global(.media-shimmer) {
		position: absolute;
		top: 0;
		left: 0;
	}

	.post-content {
		position: absolute;
		opacity: 0;
		bottom: 0;
		left: 0;
		right: 0;
		padding: 12px;
		background: linear-gradient(
			to top,
			color-mix(in srgb, var(--ctp-crust) 90%, transparent) 95%,
			transparent 100%
		);
		pointer-events: none;
		transition: opacity 0.2s;
	}

	.post-title {
		font-size: 16px;
		font-weight: 600;
		color: var(--ctp-text);
		margin-bottom: 12px;
	}

	.post-author {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.author-avatar {
		width: 28px;
		height: 28px;
		border-radius: 50%;
		object-fit: cover;
	}

	.author-info {
		flex: 1;
	}

	.author-name {
		font-weight: 500;
		color: var(--ctp-text);
		font-size: 11px;
	}

	.author-handle {
		color: var(--ctp-subtext0);
		font-size: 10px;
	}

	.post-actions {
		display: flex;
		gap: 6px;
		align-items: center;
	}

	.favourite-button {
		padding: 6px 8px;
		border-radius: var(--radius-sm);
		border: var(--border-md) solid var(--ctp-surface0);
		background: transparent;
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 4px;
		cursor: pointer;
		transition: all 0.2s;
		font-size: 0.75rem;
		flex-shrink: 0;
		color: var(--ctp-text);
	}

	.favourite-button:hover {
		background: var(--ctp-surface0);
	}

	.favourite-button.favourited {
		background: var(--ctp-yellow);
		border-color: var(--ctp-yellow);
		color: var(--ctp-crust);
	}

	.favourite-count {
		font-size: 11px;
	}
</style>
