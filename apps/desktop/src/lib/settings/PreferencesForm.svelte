<script lang="ts">
	import { GitConfigService } from '$lib/backend/gitConfigService';
	import { Project, ProjectService } from '$lib/backend/projects';
	import SectionCard from '$lib/components/SectionCard.svelte';
	import SectionCardDisclaimer from '$lib/components/SectionCardDisclaimer.svelte';
	import { projectRunCommitHooks } from '$lib/config/config';
	import Select from '$lib/select/Select.svelte';
	import SelectItem from '$lib/select/SelectItem.svelte';
	import Section from '$lib/settings/Section.svelte';
	import InfoMessage from '$lib/shared/InfoMessage.svelte';
	import Link from '$lib/shared/Link.svelte';
	import TextBox from '$lib/shared/TextBox.svelte';
	import Toggle from '$lib/shared/Toggle.svelte';
	import { getContext } from '$lib/utils/context';
	import Button from '@gitbutler/ui/Button.svelte';
	import { invoke } from '@tauri-apps/api/tauri';
	import { onMount } from 'svelte';

	const projectService = getContext(ProjectService);
	const project = getContext(Project);

	let snaphotLinesThreshold = project?.snapshot_lines_threshold || 20; // when undefined, the default is 20
	let allowForcePushing = project?.ok_with_force_push;
	let omitCertificateCheck = project?.omit_certificate_check;
	let useNewLocking = project?.use_new_locking || false;
	let ignoreProjectSemaphore = project?.ignore_project_semaphore || false;
	let signCommits = false;

	const gitConfig = getContext(GitConfigService);
	const runCommitHooks = projectRunCommitHooks(project.id);

	async function setWithForcePush(value: boolean) {
		project.ok_with_force_push = value;
		await projectService.updateProject(project);
	}

	async function setOmitCertificateCheck(value: boolean | undefined) {
		project.omit_certificate_check = !!value;
		await projectService.updateProject(project);
	}

	async function setSnapshotLinesThreshold(value: number) {
		project.snapshot_lines_threshold = value;
		await projectService.updateProject(project);
	}

	async function setSignCommits(targetState: boolean) {
		signCommits = targetState;
		await gitConfig.setGbConfig(project.id, { signCommits: targetState });
	}

	// gpg.format
	let signingFormat = 'openpgp';
	// user.signingkey
	let signingKey = '';
	// gpg.ssh.program / gpg.program
	let signingProgram = '';

	const signingFormatOptions = [
		{
			label: 'GPG',
			value: 'openpgp'
		},
		{
			label: 'SSH',
			value: 'ssh'
		}
	];

	let checked = false;
	let loading = true;
	let signCheckResult = false;
	let errorMessage = '';
	let succeedingRebases = project.succeedingRebases;

	$: {
		project.succeedingRebases = succeedingRebases;
		projectService.updateProject(project);
	}

	async function checkSigning() {
		checked = true;
		loading = true;
		await invoke('check_signing_settings', { id: project.id })
			.then((_) => {
				signCheckResult = true;
			})
			.catch((err) => {
				console.error('Error checking signing:', err);
				console.log(err.message);
				errorMessage = err.message;
				signCheckResult = false;
			});
		loading = false;
	}

	async function updateSigningInfo() {
		let signUpdate = {
			signingFormat: signingFormat,
			signingKey: signingKey,
			gpgProgram: signingFormat === 'openpgp' ? signingProgram : '',
			gpgSshProgram: signingFormat === 'ssh' ? signingProgram : ''
		};
		await gitConfig.setGbConfig(project.id, signUpdate);
	}

	async function setUseNewLocking(value: boolean) {
		project.use_new_locking = value;
		await projectService.updateProject(project);
	}
	async function setIgnoreProjectSemaphore(value: boolean) {
		project.ignore_project_semaphore = value;
		await projectService.updateProject(project);
	}

	$: setUseNewLocking(useNewLocking);
	$: setIgnoreProjectSemaphore(ignoreProjectSemaphore);

	onMount(async () => {
		let gitConfigSettings = await gitConfig.getGbConfig(project.id);
		signCommits = gitConfigSettings.signCommits || false;
		signingFormat = gitConfigSettings.signingFormat || 'openpgp';
		signingKey = gitConfigSettings.signingKey || '';
		if (signingFormat === 'openpgp') {
			signingProgram = gitConfigSettings.gpgProgram || '';
		} else {
			signingProgram = gitConfigSettings.gpgSshProgram || '';
		}
	});

	async function handleSignCommitsClick(event: MouseEvent) {
		await setSignCommits((event.target as HTMLInputElement)?.checked);
	}

	async function handleAllowForcePushClick(event: MouseEvent) {
		await setWithForcePush((event.target as HTMLInputElement)?.checked);
	}

	async function handleOmitCertificateCheckClick(event: MouseEvent) {
		await setOmitCertificateCheck((event.target as HTMLInputElement)?.checked);
	}
</script>

<Section spacer>
	<svelte:fragment slot="title">Commit signing</svelte:fragment>
	<svelte:fragment slot="description">
		Use GPG or SSH to sign your commits so they can be verified as authentic.
	</svelte:fragment>
	<SectionCard orientation="row" labelFor="signCommits">
		<svelte:fragment slot="title">Sign commits</svelte:fragment>
		<svelte:fragment slot="caption">
			GitButler will sign commits as per your git configuration.
		</svelte:fragment>
		<svelte:fragment slot="actions">
			<Toggle id="signCommits" checked={signCommits} on:click={handleSignCommitsClick} />
		</svelte:fragment>
	</SectionCard>
	{#if signCommits}
		<SectionCard orientation="column">
			<Select
				value={signingFormat}
				options={signingFormatOptions}
				label="Signing format"
				onselect={(value) => {
					signingFormat = value;
					updateSigningInfo();
				}}
			>
				{#snippet itemSnippet({ item, highlighted })}
					<SelectItem selected={item.value === signingFormat} {highlighted}>
						{item.label}
					</SelectItem>
				{/snippet}
			</Select>

			<TextBox
				label="Signing key"
				bind:value={signingKey}
				required
				on:change={updateSigningInfo}
				placeholder="ex: /Users/bob/.ssh/id_rsa.pub"
			/>

			<TextBox
				label="Signing program (optional)"
				bind:value={signingProgram}
				on:change={updateSigningInfo}
				placeholder="ex: /Applications/1Password.app/Contents/MacOS/op-ssh-sign"
			/>

			{#if checked}
				<InfoMessage
					style={loading ? 'neutral' : signCheckResult ? 'success' : 'error'}
					filled
					outlined={false}
				>
					<svelte:fragment slot="title">
						{#if loading}
							<p>Checking signing</p>
						{:else if signCheckResult}
							<p>Signing is working correctly</p>
						{:else}
							<p>Signing is not working correctly</p>
							<pre>{errorMessage}</pre>
						{/if}
					</svelte:fragment>
				</InfoMessage>
			{/if}

			<Button style="pop" kind="solid" wide icon="item-tick" onclick={checkSigning}>
				{#if !checked}
					Test signing
				{:else}
					Re-test signing
				{/if}
			</Button>
			<SectionCardDisclaimer>
				Signing commits can allow other people to verify your commits if you publish the public
				version of your signing key.
				<Link href="https://docs.gitbutler.com/features/virtual-branches/verifying-commits"
					>Read more</Link
				> about commit signing and verification.
			</SectionCardDisclaimer>
		</SectionCard>
	{/if}
</Section>

<Section spacer>
	<svelte:fragment slot="title">Preferences</svelte:fragment>
	<svelte:fragment slot="description">
		Other settings to customize your GitButler experience.
	</svelte:fragment>

	<SectionCard orientation="row" labelFor="allowForcePush">
		<svelte:fragment slot="title">Allow force pushing</svelte:fragment>
		<svelte:fragment slot="caption">
			Force pushing allows GitButler to override branches even if they were pushed to remote.
			GitButler will never force push to the target branch.
		</svelte:fragment>
		<svelte:fragment slot="actions">
			<Toggle
				id="allowForcePush"
				checked={allowForcePushing}
				on:click={handleAllowForcePushClick}
			/>
		</svelte:fragment>
	</SectionCard>

	<SectionCard orientation="row" labelFor="omitCertificateCheck">
		<svelte:fragment slot="title">Ignore host certificate checks</svelte:fragment>
		<svelte:fragment slot="caption">
			Enabling this will ignore host certificate checks when authenticating with ssh.
		</svelte:fragment>
		<svelte:fragment slot="actions">
			<Toggle
				id="omitCertificateCheck"
				checked={omitCertificateCheck}
				on:click={handleOmitCertificateCheckClick}
			/>
		</svelte:fragment>
	</SectionCard>

	<SectionCard labelFor="runHooks" orientation="row">
		<svelte:fragment slot="title">Run commit hooks</svelte:fragment>
		<svelte:fragment slot="caption">
			Enabling this will run any git pre and post commit hooks you have configured in your
			repository.
		</svelte:fragment>
		<svelte:fragment slot="actions">
			<Toggle id="runHooks" bind:checked={$runCommitHooks} />
		</svelte:fragment>
	</SectionCard>

	<SectionCard orientation="row" centerAlign>
		<svelte:fragment slot="title">Snapshot lines threshold</svelte:fragment>
		<svelte:fragment slot="caption">
			The number of lines that trigger a snapshot when saving.
		</svelte:fragment>

		<svelte:fragment slot="actions">
			<TextBox
				type="number"
				width={100}
				textAlign="center"
				value={snaphotLinesThreshold?.toString()}
				minVal={5}
				maxVal={1000}
				showCountActions
				on:change={(e) => {
					setSnapshotLinesThreshold(parseInt(e.detail));
				}}
			/>
		</svelte:fragment>
	</SectionCard>

	<SectionCard labelFor="useNewLocking" orientation="row">
		<svelte:fragment slot="title">Use new experimental hunk locking algorithm</svelte:fragment>
		<svelte:fragment slot="caption">
			This new hunk locking algorithm is still in the testing phase but should more accurately catch
			locks and subsequently cause fewer errors.
		</svelte:fragment>
		<svelte:fragment slot="actions">
			<Toggle id="useNewLocking" bind:checked={useNewLocking} />
		</svelte:fragment>
	</SectionCard>

	<SectionCard labelFor="ignoreProjectSemaphore" orientation="row">
		<svelte:fragment slot="title">Disable project semaphore usage</svelte:fragment>
		<svelte:fragment slot="caption">
			This is an experimental setting used to test if the project semaphore used in the GitButler
			app API is necessary, or if it can be removed.
		</svelte:fragment>
		<svelte:fragment slot="actions">
			<Toggle id="ignoreProjectSemaphore" bind:checked={ignoreProjectSemaphore} />
		</svelte:fragment>
	</SectionCard>

	<SectionCard labelFor="succeedingRebases" orientation="row">
		<svelte:fragment slot="title">Edit mode and succeeding rebases</svelte:fragment>
		<svelte:fragment slot="caption">
			This is an experimental setting which will ensure that rebasing will always succeed,
			introduces a mode for editing individual commits, and adds the ability to resolve conflicted
			commits.
		</svelte:fragment>
		<svelte:fragment slot="actions">
			<Toggle id="succeedingRebases" bind:checked={succeedingRebases} />
		</svelte:fragment>
	</SectionCard>
</Section>
