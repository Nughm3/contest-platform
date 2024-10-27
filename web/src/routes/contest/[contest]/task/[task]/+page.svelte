<script lang="ts">
	import Article from '$lib/components/Article.svelte';
	import { EventSourceParserStream } from 'eventsource-parser/stream';
	import { page } from '$app/stores';
	import type { Message, Verdict } from '$lib/judge/schema';
	import type { PageData } from './$types';

	interface Props {
		data: PageData;
	}

	let { data }: Props = $props();

	let formElement: HTMLFormElement | undefined = $state();
	let loading = $state(false);

	let status: string | undefined = $state();
	let tests: number | undefined = $state();
	let compileExitCode: number | undefined = $state();
	let compileStderr: string | undefined = $state();
	let progress = $state(0);
	let lastVerdict: Verdict | undefined = $state();
	let judgeError: string | undefined = $state();

	async function onsubmit() {
		const response = await fetch($page.url, {
			method: 'POST',
			body: new FormData(formElement)
		});

		if (!response.ok) {
			judgeError = await response.text();
			return;
		}

		loading = true;
		status = 'Queued';
		progress = 0;
		tests = compileExitCode = compileStderr = lastVerdict = judgeError = undefined;

		const reader = response
			.body!.pipeThrough(new TextDecoderStream())
			.pipeThrough(new EventSourceParserStream())
			.getReader();

		while (true) {
			const { value, done } = await reader.read();
			if (done) break;

			const message: Message = JSON.parse(value.data);
			status = message.type;

			switch (message.type) {
				case 'Queued':
					tests = message.tests;
					break;
				case 'Compiled':
					if (message.exit_code !== 0) {
						compileExitCode = message.exit_code;
						compileStderr = message.stderr;
					}
					break;
				case 'Judging':
					progress++;
					lastVerdict = message.verdict;
					break;
				case 'Error':
					judgeError = message.reason;
					break;
			}
		}

		loading = false;
	}
</script>

<Article title={data.name} page={data.page} />

<hr class="my-4" />

<section id="submit">
	<p class="mb-4 text-3xl font-bold">Submit</p>

	{#if judgeError}
		<p>Judge internal error</p>
		<p>{judgeError}</p>
		<p>This is not a problem with your code. Please ask for technical support.</p>
	{/if}

	{#if !loading}
		<form {onsubmit} bind:this={formElement}>
			<select name="language" id="language">
				{#each data.languages as language}
					<option>{language}</option>
				{/each}
			</select>

			<input type="file" name="code" id="code" />
			<input type="submit" value="Submit" />
		</form>
	{:else}
		{#if compileExitCode}
			<div>
				<span>Compilation exited with code {compileExitCode}</span>

				<div>
					<span>Compiler output</span>
					<pre><code>{compileStderr}</code></pre>
				</div>
			</div>
		{/if}

		{#if status}
			<span>{status.toUpperCase()}</span>
		{/if}

		{#if lastVerdict}
			<span>{lastVerdict}</span>
		{/if}

		<progress max={tests} value={progress}></progress>
	{/if}
</section>
