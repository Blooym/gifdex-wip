<script lang="ts">
	import Shimmer from '$lib/components/base/Shimmer.svelte';
	import MasonryGrid from '$lib/components/layouts/MasonryGrid.svelte';
	import { ListVideoIcon } from 'lucide-svelte';
</script>

<div class="content-container">
	<section class="hero">
		<ListVideoIcon size={42} class="logo" />
		<h1 class="site-title">Gifdex</h1>
		<p class="site-subtitle">Find and share GIFs, powered by the open web.</p>
	</section>

	<section class="trending">
		<div class="popular-gifs">
			<h2>Top Gifs</h2>
			<MasonryGrid>
				<Shimmer height={350} width={300} radius={8} />
				<Shimmer height={350} width={300} radius={8} />
				<Shimmer height={350} width={300} radius={8} />
				<Shimmer height={350} width={300} radius={8} />
				<Shimmer height={350} width={300} radius={8} />
				<Shimmer height={350} width={300} radius={8} />
				<Shimmer height={350} width={300} radius={8} />
				<Shimmer height={350} width={300} radius={8} />
				<Shimmer height={350} width={300} radius={8} />
				<Shimmer height={350} width={300} radius={8} />
				<Shimmer height={350} width={300} radius={8} />
				<Shimmer height={350} width={300} radius={8} />
			</MasonryGrid>
		</div>
		<div class="popular-tags">
			<h2>Popular tags</h2>
			<MasonryGrid>
				<Shimmer height={50} width={250} radius={8} />
				<Shimmer height={50} width={220} radius={8} />
				<Shimmer height={50} width={280} radius={8} />
				<Shimmer height={50} width={300} radius={8} />
				<Shimmer height={50} width={240} radius={8} />
				<Shimmer height={50} width={300} radius={8} />
				<Shimmer height={50} width={310} radius={8} />
				<Shimmer height={50} width={180} radius={8} />
				<Shimmer height={50} width={300} radius={8} />
				<Shimmer height={50} width={250} radius={8} />
				<Shimmer height={50} width={280} radius={8} />
				<Shimmer height={50} width={300} radius={8} />
			</MasonryGrid>
		</div>
	</section>
</div>

<style>
	.content-container {
		display: flex;
		flex-direction: column;
		row-gap: 30px;

		.hero {
			display: flex;
			flex-direction: column;
			background-color: var(--ctp-mantle);
			padding: 2rem;
			border-radius: var(--radius-md);
			width: 100%;
			max-width: 1000px;
			align-self: center;
			align-items: center;

			:global(.logo) {
				color: var(--ctp-mauve);
			}

			.site-title {
				font-size: 2rem;
			}
		}

		.trending {
			display: flex;
			flex-direction: column;
			row-gap: 20px;
		}
	}
</style>
