<script lang="ts">
	import type { PageData } from './$types';

	interface Props {
		data: PageData;
	}

	let { data }: Props = $props();
</script>

<ul>
	{#each data.contests as contest}
		<a href={'/contests/' + contest}><li>{contest}</li></a>
	{/each}
</ul>
