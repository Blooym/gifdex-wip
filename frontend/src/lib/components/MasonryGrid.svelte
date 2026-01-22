<script lang="ts">
	let { children } = $props();
</script>

<div class="masonry-container">
	<div class="masonry-grid">
		{@render children()}
	</div>
</div>

<style>
	.masonry-container {
		container-type: inline-size;
	}

	.masonry-grid {
		column-gap: 20px;
		column-fill: balance;
	}

	.masonry-grid :global(> *) {
		break-inside: avoid;
		margin-bottom: 20px;
		display: inline-block;
		width: 100%;
	}

	@container (min-width: 300px) {
		.masonry-grid {
			column-count: 1;
		}
	}

	@container (min-width: 600px) {
		.masonry-grid {
			column-count: 2;
		}
	}

	@container (min-width: 900px) {
		.masonry-grid {
			column-count: 3;
		}
	}

	@container (min-width: 1200px) {
		.masonry-grid {
			column-count: 4;
		}
	}
</style>
