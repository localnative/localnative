<script>
	import '../app.css';
	import { page } from '$app/stores';
	import {
		faNoteSticky,
		faCircleExclamation,
		faBars,
		faCloudArrowDown
	} from '@fortawesome/free-solid-svg-icons';
	import Fa from 'svelte-fa';
</script>

<div class="flex h-full w-full flex-row ">
	<div class="h-full flex flex-col z-20">
		<div>
			<ul class="menu bg-base-100 p-2">
				<li class="tooltip tooltip-right" data-tip="便签">
					<a
						href="/notes"
						class="flex justify-center items-center {$page.url.pathname == '/notes'
							? 'active'
							: ''}"
					>
						<Fa icon={faNoteSticky} size="1.4x" />
					</a>
				</li>
				<li class="tooltip tooltip-right my-1" data-tip="同步">
					<a
						href="/sync"
						class="flex justify-center items-center {$page.url.pathname == '/sync' ? 'active' : ''}"
					>
						<Fa icon={faCloudArrowDown} size="1.4x" />
					</a>
				</li>
			</ul>
		</div>
		<div class="flex-1" />
		<div>
			<ul class="menu bg-base-100 p-2">
				<li class="tooltip tooltip-right my-1" data-tip="关于">
					<a
						href="/about"
						class="flex justify-center items-center {$page.url.pathname == '/about'
							? 'active'
							: ''}"
					>
						<Fa icon={faCircleExclamation} size="1.4x" />
					</a>
				</li>
				<li class="tooltip tooltip-right" data-tip="设置">
					<a
						href="/settings"
						class="flex justify-center items-center {$page.url.pathname == '/settings'
							? 'active'
							: ''}"
					>
						<Fa icon={faBars} size="1.4x" />
					</a>
				</li>
			</ul>
		</div>
	</div>

	<div class="h-full flex-1 p-2">
		<slot />
	</div>
</div>
