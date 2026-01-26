<script lang="ts">
	import * as DropdownMenu from '$lib/components/base/dropdown-menu';
	import Shimmer from '$lib/components/base/Shimmer.svelte';
	import { authStore } from '$lib/stores/auth.svelte';
	import type { User } from '$lib/stores/user.svelte';
	import { ChevronDown, CircleUser, LogOut, Plus, User as UserIcon } from 'lucide-svelte';

	const inactiveUsers = $derived(
		authStore.users.filter((u) => u.did !== authStore.activeUser?.did)
	);

	function handleAddAccount() {
		authStore.showSignInDialog = true;
	}

	async function handleSwitchAccount(user: User) {
		await authStore.switchUser(user.did);
	}

	async function handleSignOut(user: User) {
		await authStore.signOut(user.did);
	}
</script>

<DropdownMenu.Root>
	<DropdownMenu.Trigger>
		<div class="trigger-content">
			{#if authStore.activeUser?.isLoadingProfile && !authStore.activeUser?.profile}
				<Shimmer width={28} height={28} radius="circle" />
				<Shimmer width={80} height={14} class="handle-shimmer" />
			{:else if authStore.activeUser?.profile?.avatar}
				<img
					src={authStore.activeUser.profile.avatar}
					alt={`The profile avatar of ${authStore.activeUser.profile?.displayName ?? authStore.activeUser.profile?.handle ?? authStore.activeUser.profile.did}`}
					class="avatar"
				/>
				<span class="handle"
					>@{authStore.activeUser.profile.handle ?? authStore.activeUser.did}</span
				>
			{:else}
				<div class="avatar-fallback">
					<CircleUser size={28} />
				</div>
				<span class="handle"
					>@{authStore.activeUser?.profile?.handle ?? authStore.activeUser?.did}</span
				>
			{/if}
			<ChevronDown size={16} class="chevron" />
		</div>
	</DropdownMenu.Trigger>

	<DropdownMenu.Content>
		<DropdownMenu.Section>
			<DropdownMenu.Link href="/profile/{authStore.activeUser!.did}">
				<UserIcon size={16} />
				<span>Profile</span>
			</DropdownMenu.Link>
		</DropdownMenu.Section>

		{#if inactiveUsers.length > 0}
			<DropdownMenu.Divider />
			<DropdownMenu.Section label="Switch Account">
				{#each inactiveUsers as user (user.did)}
					<div class="account-row">
						<DropdownMenu.Button onclick={() => handleSwitchAccount(user)}>
							{#if user.profile?.avatar}
								<img
									src={user.profile.avatar}
									alt={`The profile avatar of ${user.profile?.displayName ?? user.profile?.handle ?? user.profile.did}`}
									class="small-avatar"
								/>
							{:else}
								<div class="small-avatar-fallback">
									<CircleUser size={20} />
								</div>
							{/if}
							<span class="account-name">{user.profile?.handle ?? user.did}</span>
						</DropdownMenu.Button>
						<DropdownMenu.Button
							autoClose={false}
							variant="danger"
							grow={false}
							onclick={() => handleSignOut(user)}
						>
							<LogOut size={14} />
						</DropdownMenu.Button>
					</div>
				{/each}
			</DropdownMenu.Section>
		{/if}

		<DropdownMenu.Divider />
		<DropdownMenu.Section>
			<DropdownMenu.Button onclick={handleAddAccount}>
				<Plus size={16} />
				<span>Add account</span>
			</DropdownMenu.Button>
			<DropdownMenu.Button variant="danger" onclick={() => handleSignOut(authStore.activeUser!)}>
				<LogOut size={16} />
				<span>Sign out</span>
			</DropdownMenu.Button>
		</DropdownMenu.Section>
	</DropdownMenu.Content>
</DropdownMenu.Root>

<style>
	.trigger-content {
		display: contents;
	}

	.avatar {
		width: 28px;
		height: 28px;
		border-radius: 50%;
		object-fit: cover;
	}

	.avatar-fallback {
		width: 28px;
		height: 28px;
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--ctp-subtext0);
	}

	.handle,
	:global(.handle-shimmer) {
		font-size: 0.875rem;
		font-weight: 500;
		max-width: 150px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	@media (max-width: 768px) {
		.handle,
		:global(.handle-shimmer) {
			display: none;
		}
	}

	.trigger-content :global(.chevron) {
		color: var(--ctp-subtext0);
	}

	/* Account switching */
	.account-row {
		display: flex;
		gap: 4px;
	}

	.account-name {
		max-width: 200px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.small-avatar {
		width: 20px;
		height: 20px;
		border-radius: 50%;
		object-fit: cover;
		flex-shrink: 0;
	}

	.small-avatar-fallback {
		width: 20px;
		height: 20px;
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--ctp-subtext0);
		flex-shrink: 0;
	}
</style>
