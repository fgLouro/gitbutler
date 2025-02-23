<script lang="ts">
	import TextBox from '../shared/TextBox.svelte';
	import { PromptService } from '$lib/backend/prompt';
	import { getContext } from '$lib/utils/context';
	import Button from '@gitbutler/ui/Button.svelte';
	import Modal from '@gitbutler/ui/Modal.svelte';

	const promptService = getContext(PromptService);
	const [prompt, error] = promptService.reactToPrompt({ timeoutMs: 30000 });

	let value = '';
	let modal: Modal;
	let loading = false;

	$: if ($prompt) {
		modal?.show();
	}

	$: if (!$prompt && !$error) {
		modal?.close();
	}

	async function submit() {
		if (!$prompt) return;
		loading = true;
		try {
			$prompt.respond(value);
		} catch (err) {
			console.error(err);
		} finally {
			loading = false;
			clear();
		}
	}

	async function cancel() {
		try {
			if ($prompt) $prompt.respond(null);
		} catch (err) {
			console.error(err);
		} finally {
			clear();
		}
	}

	function clear() {
		prompt.set(undefined);
		error.set(undefined);
		value = '';
	}
</script>

<Modal
	bind:this={modal}
	width="small"
	title="Git fetch requires input"
	onclose={async () => await cancel()}
>
	<div class="message">
		{#if $error}
			{$error}
		{:else}
			<code>{$prompt?.prompt}</code>
		{/if}
	</div>
	<TextBox focus type="password" bind:value disabled={!!$error || loading} />

	{#snippet controls()}
		<Button style="ghost" outline type="reset" disabled={loading} onclick={cancel}>Cancel</Button>
		<Button
			style="pop"
			kind="solid"
			grow
			disabled={!!$error || loading}
			{loading}
			onclick={async () => await submit()}
		>
			Submit
		</Button>
	{/snippet}
</Modal>

<style lang="postcss">
	.message {
		padding-bottom: 12px;
	}
</style>
