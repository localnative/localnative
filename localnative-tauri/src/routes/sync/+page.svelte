<script lang="ts">
	import { open } from '@tauri-apps/plugin-dialog';
	import { listen } from '@tauri-apps/api/event';
	import { cmdServer, cmdServerPairing, cmdServerStop, cmdClientSync, cmdSyncViaAttach } from '../cmd';
	import QRCode from 'qrcode';
	import Fa from 'svelte-fa';
	import { faRotate } from '@fortawesome/free-solid-svg-icons';
	import LL from '../../i18n/i18n-svelte';
	import { onDestroy, onMount } from 'svelte';

	let syncAsClientAddr: string = '';
	let pairingCode: string = '';
	let syncing: boolean = false;
	let serverIsServing = globalThis.SyncServerOn ?? false;
	let serverAddr: string = '';
	let serverPairingCode: string = '';
	let inputInvalidAddr: boolean = false;
	let statusMessage: string = '';
	let statusIsError: boolean = false;

	// `host:port` — IPv4, IPv6 in brackets, or a DNS name; any port.
	const ADDR_RE = /^(\[[0-9a-fA-F:]+\]|[A-Za-z0-9_.-]+):(\d{1,5})$/;

	let unlisten: (() => void) | undefined;

	onMount(async () => {
		unlisten = await listen<{
			ok?: string;
			error?: string;
			server?: string;
			addresses?: string[];
			pairingCode?: string;
		}>('syncStatus', (event) => {
			syncing = false;
			const p = event.payload;
			if (p.error) {
				statusMessage = p.error;
				statusIsError = true;
				return;
			}
			statusIsError = false;
			if (p.ok) {
				statusMessage = p.ok;
			}
			if (p.server === 'started') {
				serverIsServing = true;
				globalThis.SyncServerOn = true;
				serverAddr = p.addresses?.[0] ?? '';
				serverPairingCode = p.pairingCode ?? '';
				if (serverAddr) {
					QRCode.toCanvas(document.getElementById('sync_server_qrcode'), serverAddr, {
						width: 160
					});
				}
			} else if (p.server === 'stopped') {
				serverIsServing = false;
				globalThis.SyncServerOn = false;
				serverPairingCode = '';
				serverAddr = '';
				statusMessage = '';
			}
		});
	});

	onDestroy(() => unlisten?.());

	const syncWithAttachFile = async () => {
		const selected = await open({
			title: 'Select SQLite3 Database File',
			directory: false,
			multiple: false,
			filters: [{ name: 'SQLite3 Database', extensions: ['sqlite3'] }]
		});

		if (selected != null && typeof selected == 'string') {
			cmdSyncViaAttach(selected);
		}
	};

	const syncAsClient = () => {
		if (syncing) return;

		const addr = syncAsClientAddr.trim();
		const match = ADDR_RE.exec(addr);
		const portOk = match ? Number(match[2]) <= 65535 && Number(match[2]) > 0 : false;
		if (!portOk) {
			inputInvalidAddr = true;
			return;
		}
		inputInvalidAddr = false;
		syncing = true;
		statusMessage = '';
		statusIsError = false;
		const code = pairingCode.trim();
		cmdClientSync(addr, code === '' ? undefined : code);
	};

	const startOrStopServer = () => {
		if (serverIsServing) {
			cmdServerStop();
		} else {
			cmdServer();
		}
	};

	const startPairing = () => {
		cmdServerPairing();
	};
</script>

<div class="w-full h-full flex flex-col justify-center items-center gap-y-2">
	<div class="flex flex-row justify-between items-center" style="width:600px">
		<div class="text-xl">{$LL.Sync.SyncWithFile()}</div>
		<button class="btn btn-sm" on:click={syncWithAttachFile}>
			{$LL.Sync.SyncWithFileSelect()}
		</button>
	</div>
	<hr class="my-8 h-px bg-gray-200 border-0 dark:bg-gray-700 w-full" />
	<div class="flex flex-row justify-between items-center" style="width:600px">
		<div class="text-xl">{$LL.Sync.SyncAsClient()}</div>
		<div class="form-control">
			<div class="input-group">
				<input
					type="text"
					bind:value={syncAsClientAddr}
					on:change={(_) => (inputInvalidAddr = false)}
					placeholder={$LL.Sync.SyncAsClientPlaceholder()}
					class="input input-bordered w-56 text-center {inputInvalidAddr
						? 'border-error'
						: 'undefined'}"
				/>
				<input
					type="text"
					bind:value={pairingCode}
					placeholder={$LL.Sync.PairingCodePlaceholder()}
					class="input input-bordered w-44 text-center"
				/>
				<button class="btn btn-square" on:click={syncAsClient}>
					<Fa icon={faRotate} spin={syncing} />
				</button>
			</div>
		</div>
	</div>
	<hr class="my-8 h-px bg-gray-200 border-0 dark:bg-gray-700 w-full" />
	<div class="flex flex-row justify-between" style="width:600px">
		<div class="text-xl">{$LL.Sync.SyncAsServer()}</div>
		<div class="flex flex-row gap-2">
			<button class="btn btn-sm" on:click={startPairing} disabled={!serverIsServing}>
				{$LL.Sync.PairNewDevice()}
			</button>
			<button class="btn btn-sm" on:click={startOrStopServer}>
				{serverIsServing ? $LL.Sync.StopSyncServer() : $LL.Sync.StartSyncServer()}
			</button>
		</div>
	</div>

	<div class="relative" style="width:600px">
		<div
			class="flex flex-row justify-between absolute top-6 text-xl w-full
			{serverIsServing ? 'visible' : 'invisible'}"
		>
			<div class="flex flex-col gap-2">
				<div>{$LL.Sync.SyncAsServerLocalAddr({ serverAddress: serverAddr })}</div>
				{#if serverPairingCode}
					<div class="text-lg">
						{$LL.Sync.PairingCodeShowing()} <span class="font-mono font-bold">{serverPairingCode}</span>
					</div>
				{/if}
			</div>
			<div><canvas id="sync_server_qrcode" class="rounded-xl"></canvas></div>
		</div>
	</div>

	{#if statusMessage}
		<div
			class="alert mt-4 {statusIsError ? 'alert-error' : 'alert-success'}"
			style="width:600px"
			role="alert"
		>
			<span class="break-all">{statusMessage}</span>
		</div>
	{/if}
</div>
