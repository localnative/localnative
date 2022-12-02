<script lang="ts">
	import { faMagnifyingGlass, faCalendar } from '@fortawesome/free-solid-svg-icons';
	import { DatePicker, localeFromDateFnsLocale } from 'date-picker-svelte';
	import Fa from 'svelte-fa';
	import { cmdSelect } from '../cmd';
	import Chart from './chart.svelte';
	import Notes from './notes.svelte';

	let date = new Date();
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
						type="text"
						placeholder="搜索便签..."
						class="input input-bordered input-sm w-full"
					/>
					<button class="btn btn-square btn-sm" on:click={cmdSelect}>
						<Fa icon={faMagnifyingGlass} />
					</button>
				</div>
			</div>
			<div class="btn-group">
				<button class="btn btn-sm">«</button>
				<button class="btn btn-sm">Page 22</button>
				<button class="btn btn-sm">»</button>
			</div>
		</div>
		<!-- Notes -->
		<div id="notes_panel" class="flex-1 overflow-auto">
			<Notes />
		</div>
	</div>
	<div id="tags">
		<div class="form-control">
			<div class="input-group">
				<input type="text" placeholder="搜索标签..." class="input input-bordered input-sm" />
				<button class="btn btn-square btn-sm">
					<Fa icon={faMagnifyingGlass} />
				</button>
			</div>
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
</style>
