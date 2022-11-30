<script lang="ts">
	import { faMagnifyingGlass, faCalendar } from '@fortawesome/free-solid-svg-icons';
	import { DatePicker, localeFromDateFnsLocale } from 'date-picker-svelte';
	import Fa from 'svelte-fa';

	let date = new Date();
</script>

<div class="w-full h-full flex flex-row gap-2">
	<div class="flex-1">
		<div class="form-control">
			<div class="input-group">
				<input type="text" placeholder="搜索便签..." class="input input-bordered input-sm w-full" />
				<div class="dropdown dropdown-end">
					<!-- svelte-ignore a11y-no-noninteractive-tabindex -->
					<!-- svelte-ignore a11y-label-has-associated-control -->
					<label tabindex="0" class="btn btn-sm rounded-none"><Fa icon={faCalendar} /></label>
					<!-- svelte-ignore a11y-no-noninteractive-tabindex -->
					<div
						tabindex="0"
						class="dropdown-content card card-compact p-2 shadow bg-white text-primary-content"
					>
						<div class="card-body flex flex-row">
							<DatePicker bind:value={date} />
							<DatePicker bind:value={date} />
						</div>
					</div>
				</div>

				<button class="btn btn-square btn-sm">
					<Fa icon={faMagnifyingGlass} />
				</button>
			</div>
		</div>
	</div>
	<div>
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
