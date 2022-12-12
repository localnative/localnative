<script lang="ts">
	import type { Locale } from 'date-picker-svelte';
	import { locale, setLocale, LL } from '../../i18n/i18n-svelte';
	import { isLocale } from '../../i18n/i18n-util';

	let languages = [
		{ locale: 'en', text: 'English' },
		{ locale: 'zh', text: '中文' }
	];

	let selected_locale: string = $locale;

	const select_locale = () => {
		if (isLocale(selected_locale)) {
			setLocale(selected_locale);
		}
	};
</script>

<div class="w-full h-full flex justify-center items-center">
	<div class="flex flex-row justify-between items-center w-96">
		<div>{$LL.Settings.Language()}</div>
		<select
			bind:value={selected_locale}
			on:change={select_locale}
			class="select select-bordered max-w-xs"
		>
			{#each languages as language}
				<option value={language.locale}>{language.text}</option>
			{/each}
		</select>
	</div>
</div>
