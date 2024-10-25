<script lang="ts">
	import Article from '$lib/components/Article.svelte';
	import { page } from '$app/stores';
	import type { PageData } from './$types';

	interface Props {
		data: PageData;
	}

	let { data }: Props = $props();
</script>

<Article title={data.name} page={data.page} />

<hr class="my-8" />

<section id="tasks">
	<p class="mb-4 text-3xl font-bold">Tasks</p>

	<ol class="ml-8 list-decimal">
		{#each data.tasks as task, i}
			<li>
				<a href={`${$page.url}/task/${i}`} class="underline">{task}</a>
			</li>
		{/each}
	</ol>
</section>
