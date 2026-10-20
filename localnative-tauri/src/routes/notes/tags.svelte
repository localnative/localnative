<script lang="ts">
	import { emit, listen, type UnlistenFn } from '@tauri-apps/api/event';
	import { onDestroy, onMount } from 'svelte';

	/** A tag facet as the core sends it. */
	type TagCount = { tag: string; count: number };

	let refreshTagsUnlistenFn: UnlistenFn | null = null;
	let lastTags: TagCount[] = [];

	onMount(async () => {
		refreshTagsUnlistenFn = await listen<{ tags: unknown }>('refreshTags', (ev) => {
			if (Array.isArray(ev.payload.tags)) {
				// Most used first; ties alphabetical so the order is stable.
				lastTags = [...(ev.payload.tags as TagCount[])].sort(
					(x, y) => y.count - x.count || x.tag.localeCompare(y.tag)
				);
			}
		});
	});

	onDestroy(() => {
		refreshTagsUnlistenFn?.();
	});
</script>

<slot>
	<div class="w-max-full flex flex-row flex-wrap">
		{#each lastTags as tag (tag.tag)}
			<div class="m-1">
				<button
					class="btn btn-sm normal-case"
					on:click={() => emit('update_search_tag', { tag: tag.tag })}
				>
					{tag.tag}
				</button>
				<span>{tag.count}</span>
			</div>
		{/each}
	</div>
</slot>
