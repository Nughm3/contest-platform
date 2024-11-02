<script lang="ts">
	import 'katex/dist/katex.min.css';
	import { page } from '$app/stores';
	import { Heading, Button } from 'flowbite-svelte';
	import Trophy from 'lucide-svelte/icons/trophy';
	import type { PageData } from './$types';

	interface Props {
		data: PageData;
	}

	let { data }: Props = $props();
</script>

<article class="prose max-w-full">
	<div class="flex flex-col space-y-4 md:flex-row md:items-center md:justify-between md:space-y-0">
		<Heading tag="h2" class="my-0">{data.contest!.name}</Heading>
		<a href={'/contest/' + data.contest!.slug + '/leaderboard'}>
			<Button color="alternative"><Trophy size="20" class="mr-2 inline-block" /> Leaderboard</Button
			>
		</a>
	</div>

	<hr class="my-4" />

	{@html data.page}
</article>

<hr class="my-8" />

<section id="tasks">
	<Heading tag="h3" class="mb-6">Tasks</Heading>

	<ol class="ml-8 list-decimal">
		{#each data.contest!.tasks as task, i}
			<li>
				<a href={`${$page.url}/task/${i + 1}`} class="underline">{task}</a>
			</li>
		{/each}
	</ol>
</section>
