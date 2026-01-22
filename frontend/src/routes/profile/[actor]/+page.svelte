<script lang="ts">
	import MasonryGrid from '$lib/components/MasonryGrid.svelte';
	import PostCard from '$lib/components/PostCard.svelte';
	import ProfileCard from '$lib/components/ProfileCard.svelte';
	import { NetGifdexFeedGetPostsByActor } from '$lib/lexicons';
	import type { PostFeedView } from '$lib/lexicons/types/net/gifdex/feed/defs';
	import { authStore } from '$lib/stores/auth.svelte';
	import { ok } from '@atcute/client';
	import type { Did } from '@atcute/lexicons';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	const INITIAL_LOADED_POSTS = 50;
	const PER_PAGE_LOADED_POSTS = 25;

	let posts = $state<PostFeedView[]>([]);
	let cursor = $state<number | undefined>(undefined);
	let loading = $state(false);
	let initialLoading = $state(true);
	let sentinel: HTMLElement | undefined = $state();

	$effect(() => {
		authStore.session;
		loadInitialPosts();
	});

	// Handle infinite scrolling by setting up an intersectionObserver
	$effect(() => {
		if (!sentinel) return;
		const observer = new IntersectionObserver(
			(entries) => {
				if (entries[0].isIntersecting && !loading && cursor) {
					loadMore();
				}
			},
			{
				threshold: 0.1,
				rootMargin: '150px'
			}
		);
		observer.observe(sentinel);
		return () => observer.disconnect();
	});

	async function loadInitialPosts() {
		initialLoading = true;
		try {
			const result = await ok(
				authStore.client.call(NetGifdexFeedGetPostsByActor, {
					params: {
						actor: data.actor as Did,
						limit: INITIAL_LOADED_POSTS
					}
				})
			);
			posts = result.feed;
			cursor = result.cursor;
		} catch (error) {
			console.error('Failed to load initial posts:', error);
		} finally {
			initialLoading = false;
		}
	}

	async function loadMore() {
		if (loading || !cursor) return;
		loading = true;

		try {
			const result = await ok(
				authStore.client.call(NetGifdexFeedGetPostsByActor, {
					params: {
						actor: data.actor as Did,
						limit: PER_PAGE_LOADED_POSTS,
						cursor
					}
				})
			);
			posts = [...posts, ...result.feed];
			cursor = result.cursor;
		} catch (error) {
			console.error('Failed to load more posts:', error);
		} finally {
			loading = false;
		}
	}
</script>

<MasonryGrid>
	<ProfileCard profile={data.profile} />
	{#if initialLoading}
		<div class="loading">Loading posts...</div>
	{:else}
		{#each posts as post (post.uri)}
			<PostCard {post} />
		{/each}
	{/if}
</MasonryGrid>

<!-- Infinite scroll  -->
{#if !initialLoading && cursor}
	<div bind:this={sentinel} class="sentinel">
		{#if loading}
			<div class="loading">Loading more posts...</div>
		{/if}
	</div>
{/if}

<style>
	.sentinel {
		height: 100px;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.loading {
		text-align: center;
		padding: 20px;
		color: var(--ctp-subtext0);
	}
</style>
