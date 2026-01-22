<script>
	import { authStore } from '$lib/stores/auth.svelte';
	import Button from '../Button.svelte';
	import Dialog from './Dialog.svelte';

	let handle = $state('');
</script>

<Dialog bind:open={authStore.promptSignIn}>
	<div style="display: flex; flex-direction: column; align-items: center; row-gap: 2rem;">
		<div style="display: flex; flex-direction: column; align-items: center; row-gap: 1rem;">
			<h2>Welcome to Gifdex</h2>
			<p>Sign in or create an account to get started</p>
		</div>
		<div style="display: flex; flex-direction: column; width:60%; row-gap: 0.5rem;">
			<div>
				<form style="display: flex; flex-direction: column; row-gap: 1rem;">
					<input
						bind:value={handle}
						placeholder="jane-smith.example.com"
						style="padding: 0.3rem 0.5rem;"
					/>
					<Button onclick={() => authStore.oauthSignIn(handle)} variant="primary" size="normal"
						>Sign In</Button
					>
				</form>
			</div>
			<small style="text-align: center;">OR</small>
			<div style="display: flex; flex-direction: column; row-gap: 1rem;">
				<Button variant="neutral" size="normal">Sign up via Bluesky</Button>
			</div>
		</div>
		<small
			>By signing in you agree to our <a target="_blank" href="/privacy">privacy policy</a>
			and
			<a target="_blank" href="/terms">terms of service</a></small
		>
	</div>
</Dialog>
