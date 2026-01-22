<script lang="ts">
	import Button from '$lib/components/Button.svelte';
	import { authStore } from '$lib/stores/auth.svelte';
	import { ComAtprotoRepoCreateRecord } from '@atcute/atproto';
	import { Download, Share2, Star } from 'lucide-svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	const post = $derived(data.post);
	const { did, rkey } = $derived(parseAtUri(post.uri));
	const formattedFavouriteCount = $derived(
		new Intl.NumberFormat('en', { notation: 'compact' }).format(post.favouriteCount)
	);
	const formattedPostCreatedAt = $derived(
		new Date(post.createdAt).toLocaleDateString('en-US', {
			year: 'numeric',
			month: 'long',
			day: 'numeric'
		})
	);

	function parseAtUri(uri: string) {
		const match = uri.match(/^at:\/\/([^/]+)\/([^/]+)\/(.+)$/);
		if (!match) throw new Error(`Invalid AT URI: ${uri}`);
		return { did: match[1], collection: match[2], rkey: match[3] };
	}

	async function toggleFavourite() {
		if (!authStore.isAuthenticated()) return;

		if (!post.viewer.favourite) {
			const response = await authStore.client.call(ComAtprotoRepoCreateRecord, {
				input: {
					collection: 'net.gifdex.feed.favourite',
					record: {
						$type: 'net.gifdex.feed.favourite',
						subject: post.uri,
						createdAt: new Date().toISOString()
					},
					repo: authStore.session!.info.sub
				}
			});
			if (response.ok) {
				post.viewer.favourite = parseAtUri(response.data.uri).rkey;
				post.favouriteCount += 1;
			}
		} else {
			// TODO: implement unfavourite
		}
	}

	async function sharePost() {
		if (navigator.share) {
			await navigator.share({
				title: post.title,
				url: window.location.href
			});
		} else {
			await navigator.clipboard.writeText(window.location.href);
		}
	}

	async function downloadMedia() {
		const a = document.createElement('a');
		a.href = post.media.fullsizeUrl;
		a.download = `${post.title}.gif`;
		a.click();
	}
</script>

<div class="post-view">
	<div class="post-container">
		<div class="media-section">
			<img src={post.media.fullsizeUrl} alt={post.media.alt || post.title} class="post-image" />
		</div>

		<div class="info-section">
			<div class="post-header">
				<h1 class="post-title">{post.title}</h1>
				<div class="post-meta">
					<span class="meta-item">
						<Star size={16} />
						{formattedFavouriteCount} favourites
					</span>
					<span class="meta-item date">{formattedPostCreatedAt}</span>
				</div>
			</div>

			{#if post.tags && post.tags.length > 0}
				<div class="tags">
					{#each post.tags as tag}
						<span class="tag">#{tag}</span>
					{/each}
				</div>
			{/if}

			<div class="actions">
				<Button
					variant={post.viewer.favourite ? 'primary' : 'neutral'}
					onclick={toggleFavourite}
					disabled={!authStore.isAuthenticated()}
					class="action-button"
				>
					<Star size={18} fill={post.viewer.favourite ? 'currentColor' : 'none'} />
					{post.viewer.favourite ? 'Favourited' : 'Favourite'}
				</Button>

				<Button variant="neutral" onclick={sharePost} class="action-button">
					<Share2 size={18} />
					Share
				</Button>

				<Button variant="neutral" onclick={downloadMedia} class="action-button">
					<Download size={18} />
					Download
				</Button>
			</div>

			<div class="divider"></div>

			<a href="/profile/{post.author.did}" class="author-link">
				{#if post.author.avatar}
					<img src={post.author.avatar} alt={post.author.displayName} class="author-avatar" />
				{/if}
				<div class="author-info">
					<div class="author-name">{post.author.displayName}</div>
					<div class="author-handle">@{post.author.handle}</div>
				</div>
			</a>
		</div>
	</div>
</div>

<style>
	.post-view {
		max-width: 1200px;
		margin: 0 auto;
		padding: 2rem 1rem;
	}

	.post-container {
		display: grid;
		grid-template-columns: 1fr 400px;
		gap: 2rem;
		background: var(--ctp-mantle);
		border-radius: 12px;
		overflow: hidden;
	}

	@media (max-width: 968px) {
		.post-container {
			grid-template-columns: 1fr;
		}
	}

	.media-section {
		display: flex;
		align-items: center;
		justify-content: center;
		background: var(--ctp-crust);
		padding: 2rem;
	}

	.post-image {
		max-width: 100%;
		max-height: 80vh;
		object-fit: contain;
		border-radius: 8px;
	}

	.info-section {
		padding: 2rem;
		display: flex;
		flex-direction: column;
		gap: 1.5rem;
	}

	.post-title {
		font-size: 1.75rem;
		font-weight: 700;
		color: var(--ctp-text);
		margin: 0 0 0.5rem 0;
	}

	.post-meta {
		display: flex;
		gap: 1rem;
		flex-wrap: wrap;
		font-size: 0.9rem;
	}

	.meta-item {
		display: flex;
		align-items: center;
		gap: 6px;
		color: var(--ctp-text);
	}

	.meta-item.date {
		color: var(--ctp-subtext0);
	}

	.tags {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
	}

	.tag {
		background: var(--ctp-surface0);
		color: var(--ctp-text);
		padding: 6px 12px;
		border-radius: 16px;
		font-size: 0.9rem;
	}

	.actions {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 12px;
	}

	.actions :global(.action-button) {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 8px;
	}

	.actions :global(.action-button:first-child) {
		grid-column: 1 / -1;
	}

	.divider {
		height: 1px;
		background: var(--ctp-surface0);
		margin: 0.5rem 0;
	}

	.author-link {
		display: flex;
		align-items: center;
		gap: 12px;
		text-decoration: none;
		color: var(--ctp-text);
		padding: 8px;
		border-radius: 8px;
		transition: background 0.2s;
	}

	.author-link:hover {
		background: var(--ctp-surface0);
	}

	.author-avatar {
		width: 48px;
		height: 48px;
		border-radius: 50%;
		object-fit: cover;
	}

	.author-name {
		font-weight: 600;
		font-size: 1rem;
	}

	.author-handle {
		font-size: 0.9rem;
		color: var(--ctp-subtext0);
	}
</style>
