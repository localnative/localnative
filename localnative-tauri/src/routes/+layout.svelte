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
	import { State } from './state';
	import { loadAllLocales } from '../i18n/i18n-util.sync';
	import { setLocale } from '../i18n/i18n-svelte';
	import { detectLocale, navigatorDetector } from 'typesafe-i18n/detectors';
	import LL from '../i18n/i18n-svelte';

	if (import.meta.env.PROD) {
		document.addEventListener('contextmenu', (event) => event.preventDefault());
	}

	globalThis.AppState = new State();

	loadAllLocales();
	const detectedLocale = detectLocale('en', ['en', 'zh'], navigatorDetector);
	setLocale(detectedLocale);
</script>

<div class="flex w-full flex-row h-full">
	<div id="nav" class="h-full flex flex-col z-20">
		<div>
			<ul class="menu bg-base-100 p-2">
				<li class="tooltip tooltip-right" data-tip={$LL.Nav.Notes()}>
					<a
						href="/notes"
						class="flex justify-center items-center {$page.url.pathname == '/notes'
							? 'active'
							: ''}"
					>
						<Fa icon={faNoteSticky} size="1.4x" />
					</a>
				</li>
				<li class="tooltip tooltip-right my-1" data-tip={$LL.Nav.Sync()}>
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
				<li class="tooltip tooltip-right my-1" data-tip={$LL.Nav.About()}>
					<a
						href="/about"
						class="flex justify-center items-center {$page.url.pathname == '/about'
							? 'active'
							: ''}"
					>
						<Fa icon={faCircleExclamation} size="1.4x" />
					</a>
				</li>
				<li class="tooltip tooltip-right" data-tip={$LL.Nav.Settings()}>
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
