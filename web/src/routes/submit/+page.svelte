<script lang="ts">
	import type { Message, Verdict } from '$lib/judge/schema';
	import { EventSourceParserStream } from 'eventsource-parser/stream';

	const contest = 'contest-2';
	const task = '1';
	const languages = ['C 99', 'C++ 17', 'Python 3'];

	let form: HTMLFormElement;
	let loading = false;

	let status: string | undefined;
	let tests: number | undefined;
	let compileExitCode: number | undefined;
	let compileStderr: string | undefined;
	let progress = 0;
	let lastVerdict: Verdict | undefined;
	let judgeError: string | undefined;

	async function handleSubmit() {
		const response = await fetch('/api/judge', {
			method: 'POST',
			body: new FormData(form)
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
			if (value.type !== 'event') continue;

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

{#if judgeError}
	<p>Judge internal error</p>
	<p>{judgeError}</p>
	<p>This is <em>usually</em> not a problem with your code. Please ask for technical support.</p>
{/if}

{#if !loading}
	<form on:submit|preventDefault={handleSubmit} bind:this={form}>
		<label for="language"></label>
		<select name="language" id="language" required>
			{#each languages as language}
				<option>{language}</option>
			{/each}
		</select>

		<label for="code">Upload code</label>
		<input type="file" name="code" id="code" required />

		<input type="hidden" name="contest" value={contest} />
		<input type="hidden" name="task" value={task} />

		<input type="submit" value="Submit" />
	</form>
{:else}
	{#if compileExitCode}
		<div>
			<span>Compilation exited with code {compileExitCode}</span>

			<div>
				<span>Compiler output</span>
				<pre><code> {compileStderr} </code></pre>
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
