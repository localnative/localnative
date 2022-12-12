<script lang="ts">
	import { open } from '@tauri-apps/api/dialog';
	import { invoke } from '@tauri-apps/api/tauri';
	import { cmdClientStopServer, cmdClientSync, cmdServer, cmdSyncViaAttach } from '../cmd';
	import QRCode from 'qrcode';
	import Fa from 'svelte-fa';
	import { faRotate } from '@fortawesome/free-solid-svg-icons';
	import LL from '../../i18n/i18n-svelte';
	import { onMount } from 'svelte';

	let syncAsClientAddr: string = '';
	let syncing: boolean = false;
	let serverIsServing = globalThis.SyncServerOn;
	let localIP = globalThis.LastSyncServerIp;
	let inputInvalidAddr: boolean = false;
	let syncAsClientServerAddrNotExists: boolean = false;

	onMount(async () => {
		if (serverIsServing && globalThis.LastSyncServerIp) {
			await QRCode.toCanvas(
				document.getElementById('sync_server_qrcode'),
				`${globalThis.LastSyncServerIp}:2345`,
				{
					width: 160
				}
			);
		}
	});

	const syncWithAttachFile = async () => {
		const selected = await open({
			title: 'Select SQLite3 Database File',
			directory: false,
			multiple: false,
			filters: [
				{
					name: 'SQLite3 Database',
					extensions: ['sqlite3']
				}
			]
		});

		if (selected != null && typeof selected == 'string') {
			cmdSyncViaAttach(selected);
		}
	};

	const syncAsClient = async () => {
		if (syncing) {
			return;
		}

		syncAsClientServerAddrNotExists = false;

		const regexp =
			/^(\d{1,2}|1\d\d|2[0-4]\d|25[0-5])\.(\d{1,2}|1\d\d|2[0-4]\d|25[0-5])\.(\d{1,2}|1\d\d|2[0-4]\d|25[0-5])\.(\d{1,2}|1\d\d|2[0-4]\d|25[0-5]):2345$/;

		if (!regexp.test(syncAsClientAddr)) {
			inputInvalidAddr = true;
			return;
		}

		try {
			syncing = true;
			let success = await invoke<boolean>('test_sync_server_addr', { addr: syncAsClientAddr });
			console.log('test:' + success);
			if (!success) {
				syncAsClientServerAddrNotExists = true;
				syncing = false;
				return;
			}

			cmdClientSync(syncAsClientAddr);
		} catch (err) {
			console.log('sync as client failed: ' + err);
		} finally {
			setTimeout(() => (syncing = false), 2000);
		}
	};

	const startOrStopServer = async () => {
		if (globalThis.SyncServerOn) {
			cmdClientStopServer('127.0.0.1:2345');
		} else {
			cmdServer();
			globalThis.LastSyncServerIp = await invoke<string>('local_ip');
			localIP = globalThis.LastSyncServerIp;

			await QRCode.toCanvas(
				document.getElementById('sync_server_qrcode'),
				`${globalThis.LastSyncServerIp}:2345`,
				{
					width: 160
				}
			);
		}

		serverIsServing = !serverIsServing;
		globalThis.SyncServerOn = !globalThis.SyncServerOn;
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
					class="input input-bordered w-72 text-center {inputInvalidAddr
						? 'border-error'
						: 'undefined'}"
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
		<button class="btn btn-sm" on:click={startOrStopServer}>
			{serverIsServing ? $LL.Sync.StopSyncServer() : $LL.Sync.StartSyncServer()}
		</button>
	</div>

	<div class="relative" style="width:600px">
		<div
			class="flex flex-row justify-between absolute top-6 text-xl w-full 
			{serverIsServing ? 'visible' : 'invisible'}"
		>
			<div>
				{$LL.Sync.SyncAsServerLocalAddr({ serverAddress: `${localIP}:2345` })}
			</div>
			<div><canvas id="sync_server_qrcode" class="rounded-xl" /></div>
		</div>
	</div>
</div>

<!-- <input type="checkbox" id="sync_server_addr_error_modal" class="modal-toggle" /> -->
<div class="modal {syncAsClientServerAddrNotExists ? 'modal-open' : 'undefined'}">
	<div class="modal-box">
		<h3 class="font-bold text-lg">{$LL.Sync.SyncAsClientConnectServerNotExistModalTitle()}</h3>
		<p class="py-4">{$LL.Sync.SyncAsClientConnectServerNotExistModalContent()}</p>
		<div class="modal-action">
			<!-- svelte-ignore a11y-click-events-have-key-events -->
			<!-- svelte-ignore a11y-label-has-associated-control -->
			<label class="btn" on:click={() => (syncAsClientServerAddrNotExists = false)}>
				{$LL.Ok()}
			</label>
		</div>
	</div>
</div>
