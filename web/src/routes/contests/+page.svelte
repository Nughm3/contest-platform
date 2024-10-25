<script lang="ts">
	import type { PageData } from './$types';

	export let data: PageData;
</script>

<ul>
	{#each data.contests as contest}
		<a href={'/contests/' + contest}><li>{contest}</li></a>
	{/each}
</ul>
