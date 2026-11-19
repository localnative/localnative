<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { locale, setLocale, LL } from "../../i18n/i18n-svelte";
  import { isLocale } from "../../i18n/i18n-util";
  import { cmdSearchOrFilter } from "../cmd";
  import {
    loadThemePreference,
    saveLocalePreference,
    saveThemePreference,
    type ThemePreference,
  } from "../../preferences";

  let languages = [
    { locale: "en", text: "English" },
    { locale: "zh", text: "中文" },
  ];

  let selected_locale: string = $locale;
  let theme: ThemePreference = loadThemePreference();
  let pageSize = globalThis.AppState.limit;
  let confirmDelete = globalThis.AppState.confirmDelete;
  let allowedOrigins = "";
  let browserStatus = "";

  const select_locale = () => {
    if (isLocale(selected_locale)) {
      setLocale(selected_locale);
      saveLocalePreference(selected_locale);
    }
  };

  const selectTheme = () => saveThemePreference(theme);

  const changePageSize = () => {
    globalThis.AppState.limit = pageSize;
    pageSize = globalThis.AppState.limit;
    cmdSearchOrFilter(globalThis.AppState.query);
  };

  const changeDeleteConfirmation = () => {
    globalThis.AppState.setConfirmDelete(confirmDelete);
  };

  const browserFix = async () => {
    browserStatus = "";
    const origins = allowedOrigins
      .split(",")
      .map((origin) => origin.trim())
      .filter((origin) => origin !== "");
    try {
      await invoke("fix_browser", {
        allowedOrigins: origins.length > 0 ? origins : null,
      });
      browserStatus = $LL.Settings.BrowserFixSuccess();
    } catch (error) {
      console.warn("browser host installation failed:", error);
      browserStatus = $LL.Settings.BrowserFixFailed();
    }
  };

  // Update check is manual and user-initiated: the app makes no network
  // request on its own.
  let updateStatus: string = "";
  let checking = false;

  const checkForUpdates = async () => {
    if (checking) return;
    checking = true;
    updateStatus = $LL.Settings.CheckingForUpdates();
    try {
      const { check } = await import("@tauri-apps/plugin-updater");
      const update = await check();
      if (update?.available) {
        const install = confirm(
          `A new version of Local Native is available (${update.version}). Install now?`,
        );
        if (install) {
          await update.downloadAndInstall();
          const { relaunch } = await import("@tauri-apps/plugin-process");
          await relaunch();
          return;
        }
        updateStatus = "";
      } else {
        updateStatus = $LL.Settings.UpdateUpToDate();
      }
    } catch (err) {
      console.warn("update check failed:", err);
      updateStatus = $LL.Settings.UpdateFailed();
    } finally {
      checking = false;
    }
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
      {#each languages as language (language.locale)}
        <option value={language.locale}>{language.text}</option>
      {/each}
    </select>
  </div>
  <div class="flex flex-row justify-between items-center w-96">
    <div>{$LL.Settings.Theme()}</div>
    <select
      bind:value={theme}
      on:change={selectTheme}
      class="select select-bordered max-w-xs"
    >
      <option value="system">{$LL.Settings.ThemeSystem()}</option>
      <option value="light">{$LL.Settings.ThemeLight()}</option>
      <option value="dark">{$LL.Settings.ThemeDark()}</option>
    </select>
  </div>
  <label class="flex flex-row justify-between items-center w-96">
    <span>{$LL.Settings.PageSize()}</span>
    <input
      type="number"
      min="5"
      max="1000"
      class="input input-bordered w-24"
      bind:value={pageSize}
      on:change={changePageSize}
    />
  </label>
  <label class="flex flex-row justify-between items-center w-96">
    <span>{$LL.Settings.ConfirmDelete()}</span>
    <input
      type="checkbox"
      class="toggle"
      bind:checked={confirmDelete}
      on:change={changeDeleteConfirmation}
    />
  </label>
  <label class="flex flex-col gap-2 w-96">
    <span>{$LL.Settings.AllowedOrigins()}</span>
    <input
      type="text"
      class="input input-bordered w-full"
      bind:value={allowedOrigins}
      placeholder="chrome-extension://oclkmkeameccmgnajgogjlhdjeaconnb/"
    />
  </label>
  <div class="flex flex-row justify-between items-center w-96">
    <button class="btn w-full" on:click={browserFix}
      >{$LL.Settings.BrowserFix()}</button
    >
  </div>
  {#if browserStatus}
    <div class="text-sm opacity-70" role="status">{browserStatus}</div>
  {/if}
  <div class="flex flex-row justify-between items-center w-96">
    <button class="btn w-full" on:click={checkForUpdates} disabled={checking}>
      {$LL.Settings.CheckForUpdates()}
    </button>
  </div>
  {#if updateStatus}
    <div class="text-sm opacity-70">{updateStatus}</div>
  {/if}
</div>
