<script lang="ts">
	import { emit, listen, type UnlistenFn } from '@tauri-apps/api/event';
	import { onDestroy, onMount } from 'svelte';

	let refreshTagsUnlistenFn: UnlistenFn | null = null;
	let lastTags: Array<{
		k: string;
		v: number;
	}> = [];

	onMount(async () => {
		refreshTagsUnlistenFn = await listen<any>('refreshTags', (ev) => {
			console.log('tags:' + JSON.stringify(ev.payload.tags));
			if (Array.isArray(ev.payload.tags)) {
				lastTags = ev.payload.tags;
				lastTags.sort((x, y) => y.v - x.v);
			}
		});
	});

	onDestroy(() => {
		if (refreshTagsUnlistenFn) {
			refreshTagsUnlistenFn();
		}
	});
</script>

<slot>
	<div class="w-max-full flex flex-row flex-wrap">
		{#each lastTags as tag}
			<div class="m-1">
				<button
					class="btn btn-sm normal-case"
					on:click={() => emit('update_search_tag', { tag: tag.k })}
				>
					{tag.k}
				</button>
				<span>{tag.v}</span>
			</div>
		{/each}
	</div>
</slot>
