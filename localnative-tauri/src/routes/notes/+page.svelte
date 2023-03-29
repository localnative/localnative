<script lang="ts">
	import { faMagnifyingGlass, faCalendar, faXmark } from '@fortawesome/free-solid-svg-icons';
	import { emit, listen, type UnlistenFn } from '@tauri-apps/api/event';
	import { onDestroy, onMount } from 'svelte';
	import Fa from 'svelte-fa';
	import { cmdSearchImp, cmdSearchOrFilter, cmdSelect } from '../cmd';
	import Chart from './chart.svelte';
	import Notes from './notes.svelte';
	import Tags from './tags.svelte';
	import LL from '../../i18n/i18n-svelte';
	import * as underscore from 'underscore';

	let date = new Date();

	let refreshTagsUnlistenFn: UnlistenFn | null = null;
	let updateSearchTagUnlistenFn: UnlistenFn | null = null;
	let lastPaginationText: string = '0 - 0 / 0';
	let searchText: string = '';

	onMount(async () => {
		refreshTagsUnlistenFn = await listen<any>('refreshPaginationText', (ev) => {
			console.log('pagination_text:' + JSON.stringify(ev.payload.tags));
			lastPaginationText = ev.payload.pagination_text;
		});

		updateSearchTagUnlistenFn = await listen<any>('update_search_tag', (ev) => {
			searchText = ev.payload.tag;
			search();
		});

		cmdSelect();
	});

	onDestroy(() => {
		if (refreshTagsUnlistenFn) {
			refreshTagsUnlistenFn();
		}

		if (updateSearchTagUnlistenFn) {
			updateSearchTagUnlistenFn();
		}
	});

	const search = underscore.debounce(() => {
		console.log('dddd');
		globalThis.AppState.clearOffset();
		globalThis.AppState.range = null;
		console.log('searching:' + searchText);
		cmdSearchImp(searchText);
	}, 300);

	const searchClear = () => {
		emit('resetChartZoom');
		searchText = '';
		globalThis.AppState.clearOffset();
		globalThis.AppState.range = null;
		cmdSearchImp(searchText);
	};
</script>

<div class="w-full h-full flex flex-row gap-2">
	<div class="flex-1 h-full flex flex-col overflow-hidden">
		<!-- Chart -->
		<div>
			<Chart />
		</div>

		<div class="py-5 flex flex-row justify-between gap-2">
			<div class="form-control flex-1">
				<div class="input-group">
					<input
						id="search_input"
						type="text"
						placeholder={$LL.Notes.SearchPlaceholder()}
						class="input input-bordered input-sm w-full"
						bind:value={searchText}
						on:keyup={search}
						on:input={(e) => {
							searchText = e.currentTarget.value;
							search();
						}}
					/>
					<button class="btn btn-square btn-sm" on:click={searchClear}>
						<Fa icon={faXmark} />
					</button>
					<!-- <button class="btn btn-square btn-sm" on:click={search}>
						<Fa icon={faMagnifyingGlass} />
					</button> -->
				</div>
			</div>
			<div class="btn-group">
				<button
					class="btn btn-sm"
					on:click={() => {
						globalThis.AppState.decOffset();
						cmdSearchOrFilter(searchText);
					}}
				>
					«
				</button>
				<button class="btn btn-sm">{lastPaginationText}</button>
				<button
					class="btn btn-sm"
					on:click={() => {
						globalThis.AppState.incOffset();
						cmdSearchOrFilter(searchText);
					}}
				>
					»
				</button>
			</div>
		</div>
		<!-- Notes -->
		<div id="notes_panel" class="flex-1 overflow-auto">
			<Notes current_search_text={searchText} />
		</div>
	</div>
	<div id="tags" class=" w-60 flex flex-col">
		<div class="pb-2">{$LL.Notes.Tags()}</div>
		<div id="tags_panel" class="flex-1 overflow-auto ml-2">
			<Tags />
		</div>
	</div>
</div>

<style>
	#notes_panel::-webkit-scrollbar {
		width: 14px;
	}

	#notes_panel::-webkit-scrollbar-thumb {
		border: 4px solid rgba(0, 0, 0, 0);
		background-clip: padding-box;
		border-radius: 9999px;
		background-color: #aaaaaa;
	}

	#notes_panel::-webkit-scrollbar-track {
		@apply my-4;
	}

	#tags_panel::-webkit-scrollbar {
		width: 14px;
	}

	#tags_panel::-webkit-scrollbar-thumb {
		border: 4px solid rgba(0, 0, 0, 0);
		background-clip: padding-box;
		border-radius: 9999px;
		background-color: #aaaaaa;
	}
</style>
