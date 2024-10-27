<script lang="ts">
	import type { PageData } from './$types';

	interface Props {
		data: PageData;
	}

	let { data }: Props = $props();
</script>

<ul class="ml-8 list-disc">
	{#each data.contests as [slug, name]}
		<a href={'/contest/' + slug} class="underline"><li>{name}</li></a>
	{/each}
</ul>
