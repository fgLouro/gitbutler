<script lang="ts">
	import AccountLink from '$lib/shared/AccountLink.svelte';
	import * as events from '$lib/utils/events';
	import Button from '@gitbutler/ui/Button.svelte';
	import { goto } from '$app/navigation';

	export let projectId: string | undefined;
	export let isNavCollapsed: boolean;
</script>

<div class="footer" class:collapsed={isNavCollapsed}>
	<div class="left-btns">
		<Button
			icon="mail"
			style="ghost"
			size="cta"
			help="Share feedback"
			onclick={() => events.emit('openSendIssueModal')}
			wide={isNavCollapsed}
		/>
		<Button
			icon="settings"
			style="ghost"
			size="cta"
			help="Project settings"
			onclick={async () => await goto(`/${projectId}/settings`)}
			wide={isNavCollapsed}
		/>
		<Button
			icon="timeline"
			style="ghost"
			size="cta"
			help="Project history"
			onclick={() => events.emit('openHistory')}
			wide={isNavCollapsed}
		/>
	</div>
	<AccountLink {isNavCollapsed} />
</div>

<style lang="postcss">
	.footer {
		display: flex;
		justify-content: space-between;
		padding: 12px;
		gap: 6px;
		border-top: 1px solid var(--clr-border-2);
		border-color: var(--clr-border-2);
	}

	.left-btns {
		display: flex;
		gap: 2px;
	}

	.footer.collapsed {
		flex-direction: column;
		padding: 0 14px;
		align-items: flex-start;
		gap: 4px;
		border: none;

		& .left-btns {
			flex-direction: column;
			width: 100%;
		}
	}
</style>
