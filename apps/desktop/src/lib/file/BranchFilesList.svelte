<script lang="ts">
	import BranchFilesHeader from './BranchFilesHeader.svelte';
	import FileListItem from './FileListItem.svelte';
	import LazyloadContainer from '$lib/shared/LazyloadContainer.svelte';
	import TextBox from '$lib/shared/TextBox.svelte';
	import { copyToClipboard } from '$lib/utils/clipboard';
	import { getContext } from '$lib/utils/context';
	import { selectFilesInList } from '$lib/utils/selectFilesInList';
	import { maybeMoveSelection } from '$lib/utils/selection';
	import { getCommitStore } from '$lib/vbranches/contexts';
	import { FileIdSelection, stringifyFileKey } from '$lib/vbranches/fileIdSelection';
	import { sortLikeFileTree } from '$lib/vbranches/filetree';
	import Button from '@gitbutler/ui/Button.svelte';
	import type { AnyFile } from '$lib/vbranches/types';

	export let files: AnyFile[];
	export let isUnapplied = false;
	export let showCheckboxes = false;
	export let allowMultiple = false;
	export let readonly = false;

	const fileIdSelection = getContext(FileIdSelection);
	const commit = getCommitStore();

	function chunk<T>(arr: T[], size: number) {
		return Array.from({ length: Math.ceil(arr.length / size) }, (_v, i) =>
			arr.slice(i * size, i * size + size)
		);
	}

	let chunkedFiles: AnyFile[][] = [];
	let displayedFiles: AnyFile[] = [];
	let currentDisplayIndex = 0;

	function setFiles(files: AnyFile[]) {
		chunkedFiles = chunk(sortLikeFileTree(files), 100);
		displayedFiles = chunkedFiles[0] || [];
		currentDisplayIndex = 0;
	}

	// Make sure we display when the file list is reset
	$: setFiles(files);

	export function loadMore() {
		if (currentDisplayIndex + 1 >= chunkedFiles.length) return;

		currentDisplayIndex += 1;
		const currentChunkedFiles = chunkedFiles[currentDisplayIndex] ?? [];
		displayedFiles = [...displayedFiles, ...currentChunkedFiles];
	}
	let mergeDiffCommand = 'git diff-tree --cc ';
</script>

{#if !$commit?.isMergeCommit()}
	<BranchFilesHeader title="Changed files" {files} {showCheckboxes} />
{:else}
	<div class="merge-commit-error">
		<p class="info">
			Displaying diffs for merge commits is currently not supported. Please view the merge commit in
			GitHub, or run the following command in your project directory:
		</p>
		<div class="command">
			<TextBox value={mergeDiffCommand + $commit.id.slice(0, 7)} wide readonly />
			<Button
				icon="copy"
				style="ghost"
				outline
				onmousedown={() => copyToClipboard(mergeDiffCommand + $commit.id.slice(0, 7))}
			/>
		</div>
	</div>
{/if}

{#if displayedFiles.length > 0}
	<!-- Maximum amount for initial render is 100 files
	`minTriggerCount` set to 80 in order to start the loading a bit earlier. -->
	<LazyloadContainer
		minTriggerCount={80}
		ontrigger={() => {
			console.log('loading more files...');
			loadMore();
		}}
		role="listbox"
	>
		{#each displayedFiles as file (file.id)}
			<FileListItem
				{file}
				{readonly}
				{isUnapplied}
				showCheckbox={showCheckboxes}
				selected={$fileIdSelection.includes(stringifyFileKey(file.id, $commit?.id))}
				onclick={(e) => {
					selectFilesInList(e, file, fileIdSelection, displayedFiles, allowMultiple, $commit);
				}}
				onkeydown={(e) => {
					e.preventDefault();
					maybeMoveSelection(
						{
							allowMultiple,
							shiftKey: e.shiftKey,
							key: e.key,
							targetElement: e.currentTarget as HTMLElement,
							file,
							files: displayedFiles,
							selectedFileIds: $fileIdSelection,
							fileIdSelection
						}
					);

					if (e.key === 'Escape') {
						fileIdSelection.clear();
						
						const targetEl = e.target as HTMLElement;
						targetEl.blur();
					}
				}}
			/>
		{/each}
	</LazyloadContainer>
{/if}

<style lang="postcss">
	.merge-commit-error {
		display: flex;
		flex-direction: column;
		gap: 14px;
		padding: 14px;

		& .info {
			color: var(--clr-text-2);
		}

		& .command {
			display: flex;
			gap: 8px;
			align-items: center;
			width: 100%;
		}
	}
</style>
