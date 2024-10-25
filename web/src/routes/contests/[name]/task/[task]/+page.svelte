<script lang="ts">
	import Article from '$lib/components/Article.svelte';

	import type { PageData } from './$types';

	interface Props {
		data: PageData;
	}

	let { data }: Props = $props();
</script>
