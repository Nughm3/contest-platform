<script lang="ts">
	import type { Verdict } from '$lib/judge/schema';

	interface Props {
		verdict: Verdict;
	}

	let { verdict } = $props();
</script>

{#if verdict === 'Accepted'}
	<span class="text-green-600"><strong>Accepted</strong></span>
{:else if verdict === 'Skipped'}
	<span class="text-blue-600">Skipped</span>
{:else if verdict === 'WrongAnswer'}
	<span class="text-red-600">Wrong Answer</span>
{:else if verdict === 'TimeLimitExceeded'}
	<span class="text-purple-600">Time Limit Exceeded</span>
{:else if verdict === 'MemoryLimitExceeded'}
	<span class="text-purple-600">Memory Limit Exceeded</span>
{:else if verdict === 'RuntimeError'}
	<span class="text-yellow-600">Runtime Error</span>
{:else if verdict === 'CompileError'}
	<span class="text-yellow-600">Compile Error</span>
{/if}
