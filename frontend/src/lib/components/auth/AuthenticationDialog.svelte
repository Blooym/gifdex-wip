<script>
	import { authStore } from '$lib/stores/auth.svelte';
	import Button from '../base/button/Button.svelte';
	import Dialog from '../base/dialog/Dialog.svelte';
	import Input from '../base/input/Input.svelte';
	import Link from '../base/link/Link.svelte';

	let handle = $state('');
</script>

<Dialog bind:open={authStore.showSignInDialog}>
	<div class="dialog-content">
		<div class="header">
			<h2>Sign In</h2>
			<p>Sign in or create an account to get started</p>
		</div>
		<div class="form-section">
			<form
				onsubmit={(e) => {
					e.preventDefault();
					authStore.initiateOAuthFlow(handle);
				}}
			>
				<Input required surface="mantle" bind:value={handle} placeholder="jay.bsky.social" />
				<Button type="submit" variant="primary" size="normal">Sign In</Button>
			</form>
			<span class="divider">OR</span>
			<!-- TODO: Add multiple providers to pick from. -->
			<Button
				variant="neutral"
				surface="mantle"
				size="normal"
				onclick={() => authStore.initiateOAuthFlow('https://bsky.social')}
				>Sign up via Bluesky</Button
			>
		</div>
		<small class="terms">
			By signing in you agree to our <Link target="_blank" href="/legal/privacy"
				>privacy policy</Link
			>
			and
			<Link target="_blank" href="/legal/terms">terms of service</Link>
		</small>
	</div>
</Dialog>

<style>
	.dialog-content {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 2rem;
	}

	.header {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 0.5rem;
		text-align: center;
	}

	.header h2 {
		margin: 0;
	}

	.header p {
		margin: 0;
		color: var(--ctp-subtext0);
	}

	.form-section {
		display: flex;
		flex-direction: column;
		width: min(100%, 280px);
		gap: 0.75rem;
	}

	.form-section form {
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
	}

	.divider {
		text-align: center;
		font-size: 0.75rem;
		color: var(--ctp-subtext0);
	}

	.terms {
		text-align: center;
		color: var(--ctp-subtext0);
	}
</style>
