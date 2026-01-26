<script lang="ts">
	import { Check } from 'lucide-svelte';
	import Button from '../../base/button/Button.svelte';

	let {
		close
	}: {
		close: () => void;
	} = $props();
</script>

<div class="step-complete">
	<div class="success-icon">
		<Check size={48} />
	</div>
	<h3>Your GIF has been uploaded successfully</h3>
	<p>Once we've processed it, you'll be able to find it on your profile</p>
	<div class="step-actions">
		<Button onclick={close} variant="primary">Finish</Button>
	</div>
</div>

<style>
	.step-complete {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 12px;
		min-height: 250px;

		.success-icon {
			display: flex;
			align-items: center;
			justify-content: center;
			width: 60px;
			height: 60px;
			border-radius: 50%;
			background: var(--ctp-green);
			color: var(--ctp-crust);
		}

		.step-actions {
			display: flex;
			justify-content: center;
			gap: 12px;
			margin-top: 8px;
		}
	}
</style>
