<script lang="ts">
	import Button from '$lib/components/Button.svelte';
	import { authStore } from '$lib/stores/auth.svelte';
</script>

<header>
	<div class="header-content">
		<a href={'/'} class="logo">Gifdex <small>(alpha)</small></a>
		<div class="search-container">
			<input type="text" class="search-input" placeholder="Search for GIFs or profiles..." />
		</div>
		<div class="header-actions">
			{#if authStore.isAuthenticated()}
				<Button variant="primary">Upload</Button>

				<Button
					variant="neutral"
					onclick={() => {
						authStore.oauthSignOut();
					}}>Sign Out</Button
				>
			{:else}
				<Button
					variant="neutral"
					onclick={() => {
						authStore.promptSignIn = true;
					}}>Sign in</Button
				>
			{/if}
		</div>
	</div>
</header>

<style>
	header {
		background: var(--ctp-mantle);
		border-bottom: 1px solid var(--ctp-surface0);
		padding: 16px 20px;
		width: 100%;
	}

	.header-content {
		max-width: 1400px;
		margin: 0 auto;
		display: flex;
		justify-content: space-between;
		align-items: center;
		gap: 20px;
	}

	@media (max-width: 768px) {
		.header-content {
			gap: 12px;
		}

		.search-container {
			display: none;
		}

		.logo {
			font-size: 20px;
		}
	}

	@media (max-width: 480px) {
		header {
			padding: 12px 16px;
		}
	}

	.header-actions {
		display: flex;
		gap: 12px;
		align-items: center;
	}

	.logo {
		font-size: 24px;
		font-weight: 700;
		color: var(--ctp-text);
		flex-shrink: 0;
	}

	.search-container {
		flex: 1;
		max-width: 600px;
		position: relative;
	}

	.search-input {
		width: 100%;
		padding: 10px 16px;
		border: 2px solid var(--ctp-surface0);
		border-radius: 8px;
		background: var(--ctp-base);
		color: var(--ctp-text);
		font-size: 14px;
		transition: border-color 0.2s;
	}

	.search-input:focus {
		outline: none;
		border-color: var(--ctp-mauve);
	}

	.search-input::placeholder {
		color: var(--ctp-subtext0);
	}
</style>
