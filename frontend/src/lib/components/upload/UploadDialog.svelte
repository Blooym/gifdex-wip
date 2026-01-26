<script lang="ts">
	import Dialog from '$lib/components/base/dialog/Dialog.svelte';
	import { uploadState } from './state.svelte';
	import StepComplete from './steps/StepComplete.svelte';
	import StepContentDetails from './steps/StepContentDetails.svelte';
	import StepSelectFile from './steps/StepSelectFile.svelte';
	import StepUploadProgress from './steps/StepUploadProgress.svelte';
	import { UploadStep } from './types';

	const stepTitles: Record<UploadStep, string> = {
		[UploadStep.SelectFile]: 'Create New Post',
		[UploadStep.ContentDetails]: 'Post Details',
		[UploadStep.UploadProgress]: 'Uploading',
		[UploadStep.Complete]: 'Upload Complete'
	};

	let { isOpen = $bindable(false) } = $props();

	const currentTitle = $derived(stepTitles[uploadState.currentStep]);
	const allowClose = $derived(uploadState.currentStep !== UploadStep.UploadProgress);

	$effect(() => {
		if (!isOpen) {
			uploadState.reset();
		}
	});
</script>

<Dialog bind:open={isOpen} {allowClose}>
	<div class="upload-dialog">
		<h2>{currentTitle}</h2>
		{#if uploadState.currentStep === UploadStep.SelectFile}
			<StepSelectFile />
		{:else if uploadState.currentStep === UploadStep.ContentDetails}
			<StepContentDetails />
		{:else if uploadState.currentStep === UploadStep.UploadProgress}
			<StepUploadProgress />
		{:else if uploadState.currentStep === UploadStep.Complete}
			<StepComplete close={() => (isOpen = false)} />
		{/if}
	</div>
</Dialog>

<style>
	.upload-dialog {
		display: flex;
		flex-direction: column;
		align-items: center;
		text-align: center;
	}

	h2 {
		margin-bottom: 20px;
	}
</style>
