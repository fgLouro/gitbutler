<script lang="ts">
	import Icon from '$lib/Icon.svelte';
	import { clickOutside } from '$lib/utils/clickOutside';
	import { portal } from '$lib/utils/portal';
	import type iconsJson from '$lib/data/icons.json';
	import type { Snippet } from 'svelte';

	interface Props {
		width?: 'default' | 'large' | 'small' | 'xsmall';
		title?: string;
		icon?: keyof typeof iconsJson;
		onclose?: () => void;
		children: Snippet<[item?: any]>;
		controls?: Snippet<[close: () => void, item: any]>;
	}

	const { width = 'default', title, icon, onclose, children, controls }: Props = $props();

	let item = $state<any>();
	let open = $state(false);

	export function show(newItem?: any) {
		item = newItem;
		open = true;
	}

	export function close() {
		item = undefined;
		open = false;
		onclose?.();
	}
</script>

{#if open}
	<div use:portal={'body'} role="presentation" class="modal-container" class:open>
		<div
			class="modal-content"
			class:default={width === 'default'}
			class:large={width === 'large'}
			class:small={width === 'small'}
			class:xsmall={width === 'xsmall'}
			class:round-top-corners={!title}
			use:clickOutside={{
				handler: close
			}}
		>
			{#if title}
				<div class="modal__header">
					{#if icon}
						<Icon name={icon} />
					{/if}
					<h2 class="text-14 text-semibold">
						{title}
					</h2>
				</div>
			{/if}

			<div class="modal__body custom-scrollbar text-13 text-body">
				{@render children(item)}
			</div>

			{#if controls}
				<div class="modal__footer">
					{@render controls(close, item)}
				</div>
			{/if}
		</div>
	</div>
{/if}

<style lang="postcss">
	.modal-container {
		z-index: var(--z-modal);
		position: fixed;
		top: 0;
		left: 0;
		width: 100%;
		height: 100%;

		display: flex;
		justify-content: center;
		align-items: center;

		background-color: var(--clr-overlay-bg);
	}

	.modal-container.open {
		animation: dialog-fade 0.15s ease-out;

		& .modal-content {
			animation: dialog-zoom 0.25s cubic-bezier(0.34, 1.35, 0.7, 1);
		}
	}

	.modal-content {
		display: flex;
		flex-direction: column;

		max-height: calc(100vh - 80px);
		border-radius: var(--radius-l);
		background-color: var(--clr-bg-1);
		border: 1px solid var(--clr-border-2);
		box-shadow: var(--fx-shadow-l);
		overflow: hidden;
	}

	.modal__header {
		display: flex;
		padding: 16px;
		gap: 8px;
		border-bottom: 1px solid var(--clr-border-2);
	}

	.modal__body {
		overflow: auto;
		padding: 16px;
		line-height: 160%;
	}

	.modal__body > :global(code),
	.modal__body > :global(pre) {
		word-wrap: break-word;
	}

	.modal__footer {
		display: flex;
		width: 100%;
		justify-content: flex-end;
		gap: 8px;
		padding: 16px;
		border-top: 1px solid var(--clr-border-2);
		background-color: var(--clr-bg-1);
	}

	@keyframes dialog-zoom {
		from {
			transform: scale(0.95);
		}
		to {
			transform: scale(1);
		}
	}

	@keyframes dialog-fade {
		from {
			opacity: 0;
		}
		to {
			opacity: 1;
		}
	}

	/* MODIFIERS */

	.modal-content.default {
		width: 580px;
	}

	.modal-content.large {
		width: 840px;
	}

	.modal-content.small {
		width: 380px;
	}

	.modal-content.xsmall {
		width: 310px;
	}
</style>
