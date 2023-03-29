<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import { locale, setLocale, LL } from "../../i18n/i18n-svelte";
  import { isLocale } from "../../i18n/i18n-util";

  let languages = [
    { locale: "en", text: "English" },
    { locale: "zh", text: "中文" },
  ];

  let selected_locale: string = $locale;

  const select_locale = () => {
    if (isLocale(selected_locale)) {
      setLocale(selected_locale);
    }
  };

  const browserFix = async () => {
    await invoke("fix_browser");
  };
</script>

<div class="w-full h-full flex flex-col justify-center items-center gap-4">
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
  <div class="flex flex-row justify-between items-center w-96">
    <button class="btn w-full" on:click={browserFix}>Browser Fix</button>
  </div>
</div>
