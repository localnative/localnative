<script lang="ts">
	import { open } from '@tauri-apps/api/dialog';
	import { invoke } from '@tauri-apps/api/tauri';
	import { onMount, onDestroy } from 'svelte';
	import { cmdClientStopServer, cmdClientSync, cmdServer, cmdSyncViaAttach } from '../cmd';
	import QRCode from 'qrcode';
	import Fa from 'svelte-fa';
	import { faRotate } from '@fortawesome/free-solid-svg-icons';

	let localIP: string | null = null;
	let syncAsClientAddr: string = '';
	let syncing: boolean = false;

	onMount(async () => {
		try {
			localIP = await invoke<string>('local_ip');
			await QRCode.toCanvas(document.getElementById('sync_server_qrcode'), `${localIP}:2345`, {
				width: 180
			});

			cmdServer();
		} catch (err: any) {
			console.log(`get local ip failed (${err})`);
		}
	});

	onDestroy(() => {
		cmdClientStopServer('127.0.0.1:2345');
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

	const syncAsClient = () => {
		// test ip:2345

		const regexp =
			/^(\d{1,2}|1\d\d|2[0-4]\d|25[0-5])\.(\d{1,2}|1\d\d|2[0-4]\d|25[0-5])\.(\d{1,2}|1\d\d|2[0-4]\d|25[0-5])\.(\d{1,2}|1\d\d|2[0-4]\d|25[0-5]):2345$/;

		if (!regexp.test(syncAsClientAddr)) {
			return;
		}

		try {
			syncing = true;
			cmdClientSync(syncAsClientAddr);
		} catch (err) {
			console.log('sync as client failed: ' + err);
		} finally {
			setTimeout(() => (syncing = false), 2000);
		}
	};
</script>

<div class="w-full h-full flex flex-col justify-center items-center gap-y-2">
	<div class="flex flex-row justify-between items-center" style="width:600px">
		<div class="text-xl">通过文件同步</div>
		<button class="btn btn-sm" on:click={syncWithAttachFile}>Select File</button>
	</div>
	<hr class="my-8 h-px bg-gray-200 border-0 dark:bg-gray-700 w-full" />
	<div class="flex flex-row justify-between items-center" style="width:600px">
		<div class="text-xl">作为客户端来同步</div>
		<div class="form-control">
			<div class="input-group">
				<input
					type="text"
					bind:value={syncAsClientAddr}
					placeholder="Server Address, eg: 127.0.0.1:2345"
					class="input input-bordered w-72 text-center"
				/>
				<button class="btn btn-square" on:click={syncAsClient}>
					<Fa icon={faRotate} spin={syncing} />
				</button>
			</div>
		</div>
	</div>
	<hr class="my-8 h-px bg-gray-200 border-0 dark:bg-gray-700 w-full" />
	<div class="flex flex-row justify-between" style="width:600px">
		<div class="flex flex-col text-xl">
			<div>作为服务器来同步</div>
			<div>Local Addr: {`${localIP}:2345`}</div>
		</div>
		<div><canvas id="sync_server_qrcode" class="rounded-xl" /></div>
	</div>
</div>
