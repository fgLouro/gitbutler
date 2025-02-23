<script lang="ts">
	import { Project } from '$lib/backend/projects';
	import FileMenuAction from '$lib/barmenuActions/FileMenuAction.svelte';
	import ProjectSettingsMenuAction from '$lib/barmenuActions/ProjectSettingsMenuAction.svelte';
	import { BaseBranch, NoDefaultTarget } from '$lib/baseBranch/baseBranch';
	import { BaseBranchService } from '$lib/baseBranch/baseBranchService';
	import { BranchListingService, CombinedBranchListingService } from '$lib/branches/branchListing';
	import { BranchDragActionsFactory } from '$lib/branches/dragActions';
	import { getNameNormalizationServiceContext } from '$lib/branches/nameNormalizationService';
	import { BranchService, createBranchServiceStore } from '$lib/branches/service';
	import { CommitDragActionsFactory } from '$lib/commits/dragActions';
	import EditMode from '$lib/components/EditMode.svelte';
	import NoBaseBranch from '$lib/components/NoBaseBranch.svelte';
	import NotOnGitButlerBranch from '$lib/components/NotOnGitButlerBranch.svelte';
	import ProblemLoadingRepo from '$lib/components/ProblemLoadingRepo.svelte';
	import { gitHostUsePullRequestTemplate } from '$lib/config/config';
	import { ReorderDropzoneManagerFactory } from '$lib/dragging/reorderDropzoneManager';
	import { DefaultGitHostFactory } from '$lib/gitHost/gitHostFactory';
	import { octokitFromAccessToken } from '$lib/gitHost/github/octokit';
	import { createGitHostStore } from '$lib/gitHost/interface/gitHost';
	import { createGitHostListingServiceStore } from '$lib/gitHost/interface/gitHostListingService';
	import History from '$lib/history/History.svelte';
	import { HistoryService } from '$lib/history/history';
	import MetricsReporter from '$lib/metrics/MetricsReporter.svelte';
	import { ModeService } from '$lib/modes/service';
	import Navigation from '$lib/navigation/Navigation.svelte';
	import { persisted } from '$lib/persisted/persisted';
	import { RemoteBranchService } from '$lib/stores/remoteBranches';
	import { UncommitedFilesWatcher } from '$lib/uncommitedFiles/watcher';
	import { parseRemoteUrl } from '$lib/url/gitUrl';
	import { debounce } from '$lib/utils/debounce';
	import { BranchController } from '$lib/vbranches/branchController';
	import { VirtualBranchService } from '$lib/vbranches/virtualBranch';
	import { onDestroy, setContext, type Snippet } from 'svelte';
	import type { LayoutData } from './$types';
	import { goto } from '$app/navigation';

	const { data, children }: { data: LayoutData; children: Snippet } = $props();

	const nameNormalizationService = getNameNormalizationServiceContext();

	const {
		vbranchService,
		project,
		projectId,
		projectService,
		projectMetrics,
		baseBranchService,
		remoteBranchService,
		modeService,
		userService,
		fetchSignal
	} = $derived(data);

	const branchesError = $derived(vbranchService.branchesError);
	const baseBranch = $derived(baseBranchService.base);
	const remoteUrl = $derived($baseBranch?.remoteUrl);
	const forkUrl = $derived($baseBranch?.pushRemoteUrl);
	const user = $derived(userService.user);
	const accessToken = $derived($user?.github_access_token);
	const baseError = $derived(baseBranchService.error);
	const projectError = $derived(projectService.error);

	$effect.pre(() => {
		setContext(HistoryService, data.historyService);
		setContext(VirtualBranchService, data.vbranchService);
		setContext(BranchController, data.branchController);
		setContext(BaseBranchService, data.baseBranchService);
		setContext(BaseBranch, baseBranch);
		setContext(Project, project);
		setContext(BranchDragActionsFactory, data.branchDragActionsFactory);
		setContext(CommitDragActionsFactory, data.commitDragActionsFactory);
		setContext(ReorderDropzoneManagerFactory, data.reorderDropzoneManagerFactory);
		setContext(RemoteBranchService, data.remoteBranchService);
		setContext(BranchListingService, data.branchListingService);
		setContext(ModeService, data.modeService);
		setContext(UncommitedFilesWatcher, data.uncommitedFileWatcher);
	});

	let intervalId: any;

	const showHistoryView = persisted(false, 'showHistoryView');
	const usePullRequestTemplate = gitHostUsePullRequestTemplate();
	const octokit = $derived(accessToken ? octokitFromAccessToken(accessToken) : undefined);
	const gitHostFactory = $derived(new DefaultGitHostFactory(octokit));
	const repoInfo = $derived(remoteUrl ? parseRemoteUrl(remoteUrl) : undefined);
	const forkInfo = $derived(forkUrl && forkUrl !== remoteUrl ? parseRemoteUrl(forkUrl) : undefined);
	const baseBranchName = $derived($baseBranch?.shortName);

	const listServiceStore = createGitHostListingServiceStore(undefined);
	const gitHostStore = createGitHostStore(undefined);
	const branchServiceStore = createBranchServiceStore(undefined);

	$effect.pre(() => {
		const combinedBranchListingService = new CombinedBranchListingService(
			data.branchListingService,
			listServiceStore,
			projectId
		);

		setContext(CombinedBranchListingService, combinedBranchListingService);
	});

	// Refresh base branch if git fetch event is detected.
	const mode = $derived(modeService.mode);
	const head = $derived(modeService.head);

	// We end up with a `state_unsafe_mutation` when switching projects if we
	// don't use $effect.pre here.
	// TODO: can we eliminate the need to debounce?
	const fetch = $derived(fetchSignal.event);
	const debouncedBaseBranchResfresh = debounce(() => baseBranchService.refresh(), 500);
	$effect.pre(() => {
		if ($fetch || $head) debouncedBaseBranchResfresh();
	});

	// TODO: can we eliminate the need to debounce?
	const debouncedRemoteBranchRefresh = debounce(() => remoteBranchService.refresh(), 500);
	$effect.pre(() => {
		if ($baseBranch || $head || $fetch) debouncedRemoteBranchRefresh();
	});

	$effect.pre(() => {
		const gitHost =
			repoInfo && baseBranchName
				? gitHostFactory.build(repoInfo, baseBranchName, forkInfo, usePullRequestTemplate)
				: undefined;

		const ghListService = gitHost?.listService();

		listServiceStore.set(ghListService);
		gitHostStore.set(gitHost);
		branchServiceStore.set(
			new BranchService(
				vbranchService,
				remoteBranchService,
				ghListService,
				nameNormalizationService
			)
		);
	});

	// Once on load and every time the project id changes
	$effect(() => {
		if (projectId) {
			setupFetchInterval();
		} else {
			goto('/onboarding');
		}
	});

	function setupFetchInterval() {
		baseBranchService.fetchFromRemotes();
		clearFetchInterval();
		const intervalMs = 15 * 60 * 1000; // 15 minutes
		intervalId = setInterval(async () => {
			await baseBranchService.fetchFromRemotes();
		}, intervalMs);
	}

	function clearFetchInterval() {
		if (intervalId) clearInterval(intervalId);
	}

	onDestroy(() => {
		clearFetchInterval();
	});
</script>

<!-- forces components to be recreated when projectId changes -->
{#key projectId}
	<ProjectSettingsMenuAction
		showHistory={$showHistoryView}
		onHistoryShow={(show) => ($showHistoryView = show)}
	/>
	<FileMenuAction />

	{#if !project}
		<p>Project not found!</p>
	{:else if $baseError instanceof NoDefaultTarget}
		<NoBaseBranch />
	{:else if $baseError}
		<ProblemLoadingRepo error={$baseError} />
	{:else if $branchesError}
		<ProblemLoadingRepo error={$branchesError} />
	{:else if $projectError}
		<ProblemLoadingRepo error={$projectError} />
	{:else if $baseBranch}
		{#if $mode?.type === 'OpenWorkspace'}
			<div class="view-wrap" role="group" ondragover={(e) => e.preventDefault()}>
				<Navigation />
				{#if $showHistoryView}
					<History on:hide={() => ($showHistoryView = false)} />
				{/if}
				{@render children()}
			</div>
		{:else if $mode?.type === 'OutsideWorkspace'}
			<NotOnGitButlerBranch baseBranch={$baseBranch} />
		{:else if $mode?.type === 'Edit'}
			<EditMode editModeMetadata={$mode.subject} />
		{/if}
	{/if}
	<MetricsReporter {projectMetrics} />
{/key}

<style>
	.view-wrap {
		position: relative;
		display: flex;
		width: 100%;
	}
</style>
