<script lang="ts">
	import type { ProfileView } from '$lib/lexicons/types/net/gifdex/actor/defs';
	import { CircleUserRound } from 'lucide-svelte';
	let { profile }: { profile: ProfileView } = $props();
</script>

<div class="profile-card">
	{#if profile.avatar}
		<img src={profile.avatar} alt="The avatar of {profile.handle}" loading="eager" class="avatar" />
	{:else}
		<CircleUserRound size={48} class="avatar" />
	{/if}
	{#if profile.displayName}
		<h1 class="name">{profile.displayName}</h1>
	{/if}
	{#if profile.handle}
		<p class="handle">@{profile.handle}</p>
	{:else}
		<p class="handle">@{profile.did}</p>
	{/if}
	{#if profile.pronouns}
		<p class="pronouns">{profile.pronouns}</p>
	{/if}
	{#if profile.postCount !== undefined}
		<div class="stats">
			<span class="count">{profile.postCount}</span>
			<span class="label">posts</span>
		</div>
	{/if}
</div>

<style>
	.profile-card {
		background: var(--ctp-mantle);
		border-radius: var(--radius-md);
		padding: 24px;
		text-align: center;
		break-inside: avoid;
		width: 100%;
	}

	.avatar {
		width: 80px;
		height: 80px;
		border-radius: 50%;
		object-fit: cover;
		margin: 0 auto 12px;
		display: block;
	}

	.name {
		font-size: 20px;
		font-weight: 700;
		color: var(--ctp-text);
		margin-bottom: 4px;
	}

	.handle {
		font-size: 14px;
		color: var(--ctp-subtext0);
		margin-bottom: 8px;
	}

	.pronouns {
		font-size: 12px;
		color: var(--ctp-subtext1);
		margin-bottom: 12px;
	}

	.stats {
		font-size: 14px;
		color: var(--ctp-subtext0);
		margin-top: 12px;
	}

	.count {
		font-size: 18px;
		font-weight: 700;
		display: block;
		color: var(--ctp-text);
		margin-bottom: 2px;
	}
</style>
