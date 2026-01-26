<script lang="ts">
	import Button from '$lib/components/base/button/Button.svelte';
	import { Upload, X } from 'lucide-svelte';
	import {
		ACCEPTED_TYPES,
		MAX_FILE_SIZE_BYTES,
		MAX_FILE_SIZE_MB,
		uploadState
	} from '../state.svelte';
	import { UploadStep } from '../types';

	const canContinue = $derived(uploadState.upload.file !== null);
	let error = $state('');
	let isDragging = $state(false);

	function processFile(file: File) {
		// Validate file-type again.
		// The file-picker should also filter for us.
		if (!file.type.match(/^image\/(gif|webp)$/)) {
			error = 'Please select a GIF or WebP file';
			return;
		}

		if (file.size > MAX_FILE_SIZE_BYTES) {
			error = `File size must be less than ${MAX_FILE_SIZE_MB}MB`;
			return;
		}

		error = '';

		// Generate a DataURL for the file.
		// The data will be uploaded later, and a
		// preview url will be generated automatically.
		const reader = new FileReader();
		reader.onload = (e: ProgressEvent<FileReader>) => {
			uploadState.setFile(file, e.target?.result as string | null);
		};
		reader.readAsDataURL(file);
	}

	function handleSelectFile(event: Event) {
		const target = event.target as HTMLInputElement;
		const file = target.files?.[0];

		if (!file) {
			return;
		}

		processFile(file);
	}

	function handleDragOver(event: DragEvent) {
		event.preventDefault();
		isDragging = true;
	}

	function handleDragLeave(event: DragEvent) {
		event.preventDefault();
		isDragging = false;
	}

	function handleDrop(event: DragEvent) {
		event.preventDefault();
		isDragging = false;

		const file = event.dataTransfer?.files?.[0];
		if (file) {
			processFile(file);
		}
	}

	function handleClearFile() {
		uploadState.clearFile();
	}

	function handleContinue() {
		if (canContinue) {
			uploadState.setStep(UploadStep.ContentDetails);
		}
	}
</script>

<div class="step-upload">
	{#if uploadState.upload.previewUrl}
		<div class="preview-container">
			<button
				title="Remove file"
				aria-label="Remove file"
				type="reset"
				class="clear-file"
				onclick={handleClearFile}
			>
				<X size={22} />
			</button>
			<img src={uploadState.upload.previewUrl} alt="Preview" class="preview-image" />
		</div>
	{:else}
		<label
			class="upload-zone"
			class:dragging={isDragging}
			ondragover={handleDragOver}
			ondragleave={handleDragLeave}
			ondrop={handleDrop}
		>
			<input type="file" accept="image/webp, image/gif" onchange={handleSelectFile} />
			<Upload size={64} />
			<span>Drag & drop your file here</span>
			<span>or click to browse</span>
		</label>
	{/if}
	<small class="hint"
		>Files must be under {MAX_FILE_SIZE_MB}MB. Only .{ACCEPTED_TYPES.join(', .')} can be uploaded</small
	>

	{#if error}
		<div class="error-message">{error}</div>
	{/if}

	<div class="step-actions">
		<Button onclick={handleContinue} disabled={!canContinue}>Continue</Button>
	</div>
</div>

<style>
	:root {
		--zone-max-width: 500px;
		--zone-min-height: 280px;
	}

	.step-upload {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 16px;
		width: 100%;

		.upload-zone {
			position: relative;
			display: flex;
			flex-direction: column;
			align-items: center;
			justify-content: center;
			gap: 6px;
			width: 100%;
			max-width: var(--zone-max-width);
			min-height: var(--zone-min-height);
			padding: 60px 40px;
			border: 2px dashed var(--ctp-surface0);
			border-radius: var(--radius-lg);
			background: var(--ctp-crust);
			color: var(--ctp-subtext0);
			cursor: pointer;
			transition:
				border-color 0.2s,
				background-color 0.2s;

			&:hover,
			&:has(input:focus-visible),
			&.dragging {
				border-color: var(--ctp-mauve);
				background: var(--ctp-mantle);
			}

			/* 
		  		Hide the input, but make it span the container so
		  		we can still receive browser drag and drogs.
			*/
			input[type='file'] {
				position: absolute;
				inset: 0;
				width: 100%;
				height: 100%;
				opacity: 0;
				cursor: pointer;
				z-index: 2;
			}

			/* 
		  		Ignore pointer events to avoid intercepting the 
		  		underlying drag and drop area 
			*/
			span,
			:global(svg) {
				pointer-events: none;
			}

			:global(svg) {
				color: var(--ctp-mauve);
				opacity: 0.6;
			}
		}

		.preview-container {
			position: relative;
			display: flex;
			align-items: center;
			justify-content: center;
			width: 100%;
			max-width: var(--zone-max-width);
			min-height: var(--zone-min-height);
			padding: 20px;
			border: 2px solid var(--ctp-surface0);
			border-radius: var(--radius-lg);
			background: var(--ctp-crust);

			.preview-image {
				max-width: 100%;
				max-height: 450px;
				border-radius: var(--radius-md);
				object-fit: contain;
			}

			.clear-file {
				position: absolute;
				top: 8px;
				right: 8px;
				display: flex;
				align-items: center;
				justify-content: center;
				width: 28px;
				height: 28px;
				padding: 0;
				border: none;
				border-radius: var(--radius-sm);
				background: var(--ctp-surface0);
				color: var(--ctp-subtext0);
				cursor: pointer;
				transition:
					background-color 0.2s,
					color 0.2s;
				z-index: 1;

				&:hover {
					background: var(--ctp-red);
					color: var(--ctp-crust);
				}
				&:focus-visible {
					outline: 2px solid var(--ctp-mauve);
					outline-offset: 2px;
				}
			}
		}

		.hint {
			color: var(--ctp-subtext0);
			font-size: 0.8rem;
		}

		.error-message {
			padding: 12px 16px;
			width: 100%;
			max-width: var(--zone-max-width);
			background: var(--ctp-red);
			color: var(--ctp-crust);
			border-radius: var(--radius-md);
			font-size: 0.85rem;
		}
	}
</style>
