<script lang="ts">
	import { cmdDelete, cmdUpdate, type NoteEdit } from '../cmd';
	import { emit, listen, type UnlistenFn } from '@tauri-apps/api/event';
	import { onDestroy, onMount } from 'svelte';
	import { faTrash, faQrcode, faPen } from '@fortawesome/free-solid-svg-icons';
	import Fa from 'svelte-fa';
	import QRCode from 'qrcode';
	import LL from '../../i18n/i18n-svelte';

	/** A note as the core's read queries return it. */
	type Note = {
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
	};

	export let current_search_text: string;
	let refreshNotesUnlistenFn: UnlistenFn | null = null;
	let lastNotes: Note[] = [];

	let deleteDialog: HTMLDialogElement;
	let wantDeleteRowId: number = 0;

	// The edit dialog works on a copy of the note's fields, so cancelling
	// (button, Escape) leaves the list exactly as it was.
	let editDialog: HTMLDialogElement;
	let editing: (NoteEdit & { uuid4: string }) | null = null;

	onMount(async () => {
		refreshNotesUnlistenFn = await listen<{ notes: Note[] }>('refreshNotes', (ev) => {
			lastNotes = ev.payload.notes;
		});
	});

	onDestroy(() => {
		refreshNotesUnlistenFn?.();
	});

	const drawQR = async (uuid: string, url: string) => {
		await QRCode.toCanvas(document.getElementById(`qr_${uuid}`), url);
	};

	// An untagged note has `tags === ''`, which would split into one empty tag.
	const tagsOf = (note: Note) => note.tags.split(',').filter((tag) => tag !== '');

	const askDelete = (rowid: number) => {
		wantDeleteRowId = rowid;
		deleteDialog.showModal();
	};

	const confirmDelete = () => {
		cmdDelete(current_search_text, wantDeleteRowId);
		wantDeleteRowId = 0;
		deleteDialog.close();
	};

	const startEdit = (note: Note) => {
		editing = {
			uuid4: note.uuid4,
			title: note.title,
			url: note.url,
			tags: note.tags,
			description: note.description
		};
		editDialog.showModal();
	};

	const saveEdit = () => {
		if (!editing) return;
		const { uuid4, ...edit } = editing;
		cmdUpdate(current_search_text, uuid4, edit);
		editDialog.close();
	};
</script>

<slot>
	{#each lastNotes as note (note.uuid4)}
		<div class="my-2 card card-compact bg-yellow-500 shadow-sm text-neutral break-all">
			<div class="card-body">
				<div class="flex flex-row gap-2 justify-between items-center">
					<div class="flex flex-row gap-2 items-center">
						<div class="dropdown dropdown-start">
							<!-- A focusable div, not a button: the dropdown opens on focus, and
							     WebKit does not focus a button on click. -->
							<div
								tabindex="0"
								role="button"
								class="btn btn-xs"
								on:click={() => drawQR(note.uuid4, note.url)}
								on:keydown={(e) => (e.key === 'Enter' || e.key === ' ') && drawQR(note.uuid4, note.url)}
							>
								<Fa icon={faQrcode} />
							</div>
							<div tabindex="-1" class="dropdown-content menu p-2 mt-2 shadow bg-base-100 rounded-box">
								<canvas id={`qr_${note.uuid4}`} class="rounded-xl" width="0" height="0"></canvas>
							</div>
						</div>
						<div>{note.created_at}</div>
						<div>row_id: {note.rowid}</div>
						<div>uuid: {note.uuid4.substring(0, 6) + '...'}</div>
					</div>
					<div class="flex flex-row gap-2">
						{#each tagsOf(note) as tag}
							<button
								class="btn btn-xs normal-case"
								on:click={() => emit('update_search_tag', { tag })}
							>
								{tag}
							</button>
						{/each}

						<button type="button" class="btn btn-xs" title={$LL.Notes.Edit()} on:click={() => startEdit(note)}>
							<Fa icon={faPen} />
						</button>
						<button type="button" class="btn btn-xs" title={$LL.Notes.Delete()} on:click={() => askDelete(note.rowid)}>
							<Fa icon={faTrash} />
						</button>
					</div>
				</div>
				<div>{note.title}</div>
				<a href={note.url} class="link" rel="noreferrer" target="_blank">{note.url}</a>
				<div>{note.description}</div>
			</div>
		</div>
	{/each}

	<!-- Delete confirmation -->
	<dialog class="modal" bind:this={deleteDialog} on:close={() => (wantDeleteRowId = 0)}>
		<div class="modal-box">
			<h3 class="font-bold text-lg">{$LL.Notes.DeleteModalTitle()}</h3>
			<p class="py-4">{$LL.Notes.DeleteModalContent()}</p>
			<div class="modal-action">
				<button type="button" class="btn" on:click={confirmDelete}>{$LL.Yes()}</button>
				<button type="button" class="btn" on:click={() => deleteDialog.close()}>{$LL.No()}</button>
			</div>
		</div>
	</dialog>

	<!-- Edit dialog: title, URL, tags and description go to the core's `update` -->
	<dialog class="modal" bind:this={editDialog} on:close={() => (editing = null)}>
		<div class="modal-box">
			<h3 class="font-bold text-lg">{$LL.Notes.EditModalTitle()}</h3>
			{#if editing}
				<form class="flex flex-col gap-3 py-4" on:submit|preventDefault={saveEdit}>
					<label class="flex flex-col gap-1">
						<span class="text-sm opacity-70">{$LL.Notes.EditTitle()}</span>
						<!-- svelte-ignore a11y-autofocus -->
						<input class="input input-bordered input-sm w-full" bind:value={editing.title} autofocus />
					</label>
					<label class="flex flex-col gap-1">
						<span class="text-sm opacity-70">{$LL.Notes.EditUrl()}</span>
						<input class="input input-bordered input-sm w-full" bind:value={editing.url} />
					</label>
					<label class="flex flex-col gap-1">
						<span class="text-sm opacity-70">{$LL.Notes.EditTags()}</span>
						<input class="input input-bordered input-sm w-full" bind:value={editing.tags} />
					</label>
					<label class="flex flex-col gap-1">
						<span class="text-sm opacity-70">{$LL.Notes.EditDescription()}</span>
						<textarea class="textarea textarea-bordered w-full" rows="4" bind:value={editing.description}></textarea>
					</label>
					<div class="modal-action">
						<button type="submit" class="btn btn-primary">{$LL.Notes.Save()}</button>
						<button type="button" class="btn" on:click={() => editDialog.close()}>{$LL.Notes.Cancel()}</button>
					</div>
				</form>
			{/if}
		</div>
	</dialog>
</slot>
