<script lang="ts">
	import 'katex/dist/katex.min.css';

	export let title: string;
	export let page: string;
</script>

<article class="prose">
	<h1>{title}</h1>

	{@html page}
</article>
