<script lang="ts">
	import 'katex/dist/katex.min.css';

	interface Props {
		title: string;
		page: string;
	}

	let { title, page }: Props = $props();
</script>

<article class="prose">
	<h1>{title}</h1>

	{@html page}
</article>
