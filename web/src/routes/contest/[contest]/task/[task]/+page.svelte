<script lang="ts">
	import 'katex/dist/katex.min.css';
	import { EventSourceParserStream } from 'eventsource-parser/stream';
	import { page } from '$app/stores';
	import {
		Button,
		Fileupload,
		Heading,
		Helper,
		Input,
		Label,
		Progressbar,
		Select,
		Spinner,
		Table,
		TableBody,
		TableBodyCell,
		TableBodyRow,
		TableHead,
		TableHeadCell
	} from 'flowbite-svelte';
	import { goto } from '$app/navigation';
	import Verdict from '$lib/components/Verdict.svelte';
	import type { Message, Verdict as VerdictType } from '$lib/judge/schema';
	import type { PageData, ActionData } from './$types';
	import CodeXml from 'lucide-svelte/icons/code-xml';

	interface Props {
		data: PageData;
		form: ActionData;
	}

	let { data, form }: Props = $props();

	let submissions = $state(data.submissions);

	let formElement: HTMLFormElement | undefined = $state();
	let loading = $state(false);

	let currentTime = $state(new Date().getTime());
	$effect(() => {
		const interval = setInterval(() => (currentTime = new Date().getTime()), 500);
		return () => clearInterval(interval);
	});
	let timeLeft = $derived(
		data.lastSubmissionTime != null
			? Math.max(data.submissionCooldown - (currentTime - data.lastSubmissionTime), 0)
			: 0
	);

	let status: string | undefined = $state();
	let tests: number | undefined = $state();
	let compileExitCode: number | undefined = $state();
	let compileStderr: string | undefined = $state();
	let progress = $state(0);
	let lastVerdict: VerdictType | undefined = $state();
	let judgeError: string | undefined = $state();

	async function onsubmit(event: Event) {
		event.preventDefault();
		goto('#submit');

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
				case 'Done':
					submissions.splice(0, 0, {
						score: 0,
						verdict: message.report.task,
						timestamp: new Date()
					});
					break;
			}
		}

		loading = false;
		goto('#submissions');
	}
</script>

<div class="flex flex-col space-y-4 md:flex-row md:items-center md:justify-between md:space-y-0">
	<Heading tag="h2">{data.name}</Heading>
	<a href="#submit">
		<Button color="alternative" class="flex items-center space-x-1">
			<CodeXml size="20" class="inline-block" />
			<span>Submit</span>
		</Button>
	</a>
</div>

<div class="mt-4 flex flex-col md:flex-row md:justify-between">
	<span class="mb-0">
		{data.difficulty}
		{#if data.difficulty === 'Easy'}
			&starf;&star;&star;
		{:else if data.difficulty === 'Medium'}
			&starf;&starf;&star;
		{:else if data.difficulty === 'Hard'}
			&starf;&starf;&starf;
		{/if}
	</span>
	{#if !data.answerSubmission}
		<span class="mb-0">
			Resource limits: {data.rlimits.cpu} seconds, {data.rlimits.memory / 1e6} MB
		</span>
	{/if}
</div>

<hr class="my-4" />

<article class="prose max-w-full">
	{@html data.page}
</article>

<hr class="my-8" />

<section id="submit">
	<Heading tag="h3" class="mb-6">Submit</Heading>

	{#if judgeError}
		<Helper color="red" class="text-md mb-2">
			<strong>Judge internal error:</strong>
			{judgeError}
		</Helper>
		<Helper color="red" class="mb-2">
			This is not a problem with your code. Please ask for technical support.
		</Helper>
	{/if}

	{#if compileExitCode}
		<Helper color="red" class="text-md mb-2">
			Compiler output (exited with code <strong>{compileExitCode}</strong>)
		</Helper>
		<div class="prose mb-2 max-w-full">
			<pre><code>{compileStderr}</code></pre>
		</div>
	{/if}

	{#if !loading}
		{#if data.answerSubmission}
			<form method="POST" action="?/submitAnswer" class="mb-6">
				<div
					class="flex flex-col space-y-6 md:flex-row md:items-end md:justify-between md:space-x-6 md:space-y-0"
				>
					<div class="grow">
						<Label for="answer" class="mb-2">Answer</Label>
						<Input
							type="text"
							name="answer"
							id="answer"
							placeholder="Enter your answer..."
							required
						/>
					</div>
					{#if timeLeft === 0}
						<Button type="submit">Submit</Button>
					{:else}
						<Button type="submit" disabled>{timeLeft}s</Button>
					{/if}
				</div>

				{#if form?.verdict}
					{form.verdict}
				{/if}
			</form>
		{:else}
			<form {onsubmit} bind:this={formElement} class="mb-6">
				<div
					class="flex flex-col space-y-6 md:flex-row md:items-end md:justify-between md:space-x-6 md:space-y-0"
				>
					<div class="flex-auto">
						<Label for="code" class="mb-2">Upload code</Label>
						<Fileupload name="code" id="code" required class="h-[40px]" />
					</div>

					<div class="flex-auto">
						<Label for="language" class="mb-2">Language</Label>
						<Select
							name="language"
							id="language"
							placeholder="Select language..."
							required
							class="h-[40px]"
						>
							{#each data.languages as language}
								<option>{language}</option>
							{/each}
						</Select>
					</div>

					{#if timeLeft === 0}
						<Button type="submit">Submit</Button>
					{:else}
						<Button type="submit" disabled>{timeLeft}s</Button>
					{/if}
				</div>
			</form>
		{/if}
	{:else if status}
		<div class="mb-2 flex items-center justify-between">
			<span class="font-medium"><Spinner size="4" /> {status}</span>
			{#if lastVerdict}
				<Verdict verdict={lastVerdict} />
			{/if}
		</div>

		<Progressbar progress={(progress / tests!) * 100} animate />
	{/if}
</section>

{#if submissions.length > 0}
	<hr class="my-8" />

	<section id="submissions">
		<Heading tag="h3" class="mb-6">Submissions</Heading>

		<Table>
			<TableHead>
				<TableHeadCell>Time</TableHeadCell>
				<TableHeadCell>Score</TableHeadCell>
				<TableHeadCell>Verdict</TableHeadCell>
			</TableHead>
			<TableBody>
				{#each submissions as submission}
					<TableBodyRow>
						<TableBodyCell>{submission.timestamp.toLocaleString()}</TableBodyCell>
						<TableBodyCell>{submission.score}</TableBodyCell>
						<TableBodyCell><Verdict verdict={submission.verdict} /></TableBodyCell>
					</TableBodyRow>
				{/each}
			</TableBody>
		</Table>
	</section>
{/if}
