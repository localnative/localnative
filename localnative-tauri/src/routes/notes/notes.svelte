<script lang="ts">
	import { cmdDelete } from '../cmd';
	import { emit, listen, type UnlistenFn } from '@tauri-apps/api/event';
	import { onDestroy, onMount } from 'svelte';
	import { faTrash, faQrcode } from '@fortawesome/free-solid-svg-icons';
	import Fa from 'svelte-fa';
	import QRCode from 'qrcode';
	import LL from '../../i18n/i18n-svelte';

	export let current_search_text: string;
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
	let wantDeleteRowId: number = 0;

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

	const drawQR = async (uuid: string, url: string) => {
		await QRCode.toCanvas(document.getElementById(`qr_${uuid}`), url);
	};
</script>

<!-- svelte-ignore a11y-click-events-have-key-events -->
<!-- svelte-ignore a11y-no-noninteractive-tabindex -->
<slot>
	{#each lastNotes as note}
		<div class="my-2 card card-compact bg-yellow-500 shadow-sm text-neutral break-all">
			<div class="card-body">
				<div class="flex flex-row gap-2 justify-between items-center">
					<div class="flex flex-row gap-2 items-center">
						<div class="dropdown">
							<div class="dropdown dropdown-start" on:click={() => drawQR(note.uuid4, note.url)}>
								<!-- svelte-ignore a11y-label-has-associated-control -->
								<label tabindex="0" class="btn btn-xs">
									<Fa icon={faQrcode} />
								</label>
								<div
									tabindex="0"
									class="dropdown-content menu p-2 mt-2 shadow bg-base-100 rounded-box"
								>
									<canvas id={`qr_${note.uuid4}`} class="rounded-xl" width="0" height="0" />
								</div>
							</div>
						</div>
						<div>{note.created_at}</div>
						<div>row_id: {note.rowid}</div>
						<div>uuid: {note.uuid4.substring(0, 6) + '...'}</div>
					</div>
					<div class="flex flex-row gap-2">
						{#each note.tags.split(',') as tag}
							<button
								class="btn btn-xs normal-case"
								on:click={() => emit('update_search_tag', { tag })}
							>
								{tag}
							</button>
						{/each}

						<label
							class="btn btn-xs"
							for="delete-confirm-modal"
							on:click={() => (wantDeleteRowId = note.rowid)}
						>
							<Fa icon={faTrash} />
						</label>
					</div>
				</div>
				<div>{note.title}</div>
				<a href={note.url} class="link" rel="noreferrer" target="_blank">{note.url}</a>
				<div>{note.description}</div>
			</div>
		</div>
	{/each}

	<!--Delete Confirm Modal-->
	<input type="checkbox" id="delete-confirm-modal" class="modal-toggle" />
	<div class="modal">
		<div class="modal-box">
			<h3 class="font-bold text-lg">{$LL.Notes.DeleteModalTitle()}</h3>
			<p class="py-4">{$LL.Notes.DeleteModalContent()}</p>
			<div class="modal-action">
				<label
					for="delete-confirm-modal"
					class="btn"
					on:click={() => {
						cmdDelete(current_search_text, wantDeleteRowId);
						wantDeleteRowId = 0;
					}}
				>
					{$LL.Yes()}
				</label>
				<label for="delete-confirm-modal" class="btn" on:click={() => (wantDeleteRowId = 0)}>
					{$LL.No()}
				</label>
			</div>
		</div>
	</div>
</slot>
