<script lang="ts">
	import { faFilter } from '@fortawesome/free-solid-svg-icons';
	import { emit } from '@tauri-apps/api/event';
	import Fa from 'svelte-fa';

	export let tag: { k: string; v: number };
	export let total_tags: number;

	let mouse_enter: boolean = false;
</script>

<slot>
	<tr on:mouseenter={() => (mouse_enter = true)} on:mouseleave={() => (mouse_enter = false)}>
		<td class="w-full">{tag.k}</td>
		<td class="text-center px-4">{tag.v}</td>
		<td class="text-center">
			<button
				class="btn btn-xs {total_tags == 1 || mouse_enter ? 'visible' : 'invisible'}"
				on:click={() => emit('update_search_tag', { tag: tag.k })}
			>
				<Fa icon={faFilter} />
			</button>
		</td>
	</tr>
</slot>
