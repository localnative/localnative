<script lang="ts">
	import * as d3 from 'd3';
	import * as dc from 'dc';
	import crossfilter from 'crossfilter2';
	import { cmdFilter } from '../cmd';
	import { emit, listen, type UnlistenFn } from '@tauri-apps/api/event';
	import { onDestroy, onMount } from 'svelte';
	import 'dc/src/compat/d3v6';
	import { faXmark, faQrcode } from '@fortawesome/free-solid-svg-icons';
	import Fa from 'svelte-fa';

	let chartContainer: Element;
	let refreshNotesUnlistenFn: UnlistenFn | null = null;
	let lastNotes: Array<{
		annotations: string;
		comments: string;
		created_at: string;
		description: string;
		is_public: boolean;
		rowid: number;
		tags: string;
		title: string;
		url: string;
		uuid4: string;
	}> = [];

	onMount(async () => {
		refreshNotesUnlistenFn = await listen<any>('refreshNotes', (ev) => {
			console.log('notes:' + JSON.stringify(ev.payload.notes));
			lastNotes = ev.payload.notes;
		});
	});

	onDestroy(() => {
		if (refreshNotesUnlistenFn) {
			refreshNotesUnlistenFn();
		}
	});
</script>

<slot>
	{#each lastNotes as note}
		<div class="my-2 card card-compact bg-yellow-500 shadow-sm text-neutral">
			<div class="card-body">
				<div class="flex flex-row gap-2 justify-between">
					<div class="flex flex-row gap-2">
						<div>{note.created_at}</div>
						<div>row_id: {note.rowid}</div>
						<div>uuid: {note.uuid4.substring(0, 6) + '...'}</div>
					</div>
					<div class="flex flex-row gap-2">
						<button class="btn btn-xs">{note.tags}</button>
						<div class="input-group">
							<button class="btn btn-xs"><Fa icon={faQrcode} /></button>
							<button class="btn btn-xs"><Fa icon={faXmark} /></button>
						</div>
					</div>
				</div>
				<div>{note.title}</div>
				<a href={note.url} class="link" rel="noreferrer" target="_blank">{note.url}</a>
				<div>{note.description}</div>
			</div>
		</div>
	{/each}
</slot>
