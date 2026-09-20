<script>
	import '../app.css';
	import { page } from '$app/stores';
	import {
		faNoteSticky,
		faCircleExclamation,
		faBars,
		faCopy
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

	// Tauri 2 updater: the v1 built-in update dialog is gone, so replicate it —
	// check on startup, confirm, install, relaunch. Runs only inside Tauri.
	if (import.meta.env.PROD && '__TAURI_INTERNALS__' in globalThis) {
		import('@tauri-apps/api/app')
			.then(({ getVersion }) => getVersion())
			.then((currentVersion) => {
				import('@tauri-apps/plugin-updater')
					.then(({ check }) => check())
					.then((update) => {
						if (update?.available && update.version !== currentVersion) {
							const install = confirm(
								`A new version of Local Native is available (${update.version}). Install now?`
							);
							if (install) {
								return update
									.downloadAndInstall()
									.then(() => import('@tauri-apps/plugin-process'))
									.then(({ relaunch }) => relaunch());
							}
						}
					})
					.catch((err) => console.warn('update check failed:', err));
			});
	}
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
						<Fa icon={faCopy} size="1.4x" />
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
