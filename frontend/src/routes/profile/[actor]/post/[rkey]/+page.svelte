<script lang="ts">
	import Button from '$lib/components/base/button/Button.svelte';
	import { authStore } from '$lib/stores/auth.svelte';
	import { ComAtprotoRepoCreateRecord } from '@atcute/atproto';
	import { Download, Share2, Star } from 'lucide-svelte';
	import type { PageProps } from './$types';

	const { data }: PageProps = $props();

	function parseAtUri(uri: string) {
		const match = uri.match(/^at:\/\/([^/]+)\/([^/]+)\/(.+)$/);
		if (!match) throw new Error(`Invalid AT URI: ${uri}`);
		return { did: match[1], collection: match[2], rkey: match[3] };
	}

	async function toggleFavourite() {
		if (!authStore.isAuthenticated()) return;

		if (!data.post.viewer.favourite) {
			const response = await authStore.client.call(ComAtprotoRepoCreateRecord, {
				input: {
					collection: 'net.gifdex.feed.favourite',
					record: {
						$type: 'net.gifdex.feed.favourite',
						subject: data.post.uri,
						createdAt: new Date().toISOString()
					},
					repo: authStore.activeUser.did
				}
			});
			if (response.ok) {
				data.post.viewer.favourite = parseAtUri(response.data.uri).rkey;
				data.post.favouriteCount += 1;
			}
		} else {
			// TODO: implement unfavourite
		}
	}

	async function sharePost() {
		if (navigator.share) {
			await navigator.share({
				title: data.post.title,
				url: window.location.href
			});
		} else {
			await navigator.clipboard.writeText(window.location.href);
		}
	}

	async function downloadMedia() {
		const a = document.createElement('a');
		a.href = data.post.media.fullsizeUrl;
		a.download = `${data.post.title}.gif`;
		a.click();
	}
</script>

<div class="post-view">
	<div class="post-container">
		<div class="media-section">
			<img
				src={data.post.media.fullsizeUrl}
				alt={data.post.media.alt || data.post.title}
				class="post-image"
			/>
		</div>

		<div class="info-section">
			<div class="post-header">
				<h1 class="post-title">{data.post.title}</h1>
				<div class="post-meta">
					<span class="meta-item">
						<Star size={16} />
						{new Intl.NumberFormat('en', { notation: 'compact' }).format(data.post.favouriteCount)} favourites
					</span>
					<span class="meta-item date"
						>{new Date(data.post.createdAt).toLocaleDateString('en-US', {
							year: 'numeric',
							month: 'long',
							day: 'numeric'
						})}</span
					>
				</div>
			</div>

			{#if data.post.tags && data.post.tags.length > 0}
				<div class="tags">
					{#each data.post.tags as tag}
						<span class="tag">#{tag}</span>
					{/each}
				</div>
			{/if}

			<div class="actions">
				<Button
					variant={data.post.viewer.favourite ? 'primary' : 'neutral'}
					onclick={toggleFavourite}
					disabled={!authStore.isAuthenticated()}
					class="action-button"
				>
					<Star size={18} fill={data.post.viewer.favourite ? 'currentColor' : 'none'} />
					{data.post.viewer.favourite ? 'Favourited' : 'Favourite'}
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

			<a href="/profile/{data.post.author.did}" class="author-link">
				{#if data.post.author.avatar}
					<img
						src={data.post.author.avatar}
						alt={data.post.author.displayName}
						class="author-avatar"
					/>
				{/if}
				<div class="author-info">
					<div class="author-name">{data.post.author.displayName}</div>
					<div class="author-handle">@{data.post.author.handle}</div>
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
