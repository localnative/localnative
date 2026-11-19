<script>
  import "../app.css";
  import { page } from "$app/stores";
  import { resolve } from "$app/paths";
  import {
    faNoteSticky,
    faCircleExclamation,
    faBars,
    faCopy,
  } from "@fortawesome/free-solid-svg-icons";
  import Fa from "svelte-fa";
  import { State } from "./state";
  import { loadAllLocales } from "../i18n/i18n-util.sync";
  import { isLocale } from "../i18n/i18n-util";
  import { setLocale } from "../i18n/i18n-svelte";
  import { detectLocale, navigatorDetector } from "typesafe-i18n/detectors";
  import LL from "../i18n/i18n-svelte";
  import { onDestroy } from "svelte";
  import {
    applyThemePreference,
    loadLocalePreference,
    loadThemePreference,
  } from "../preferences";

  if (import.meta.env.PROD) {
    document.addEventListener("contextmenu", (event) => event.preventDefault());
  }

  globalThis.AppState = new State();

  applyThemePreference(loadThemePreference());
  const systemTheme = window.matchMedia("(prefers-color-scheme: dark)");
  const followSystemTheme = () => {
    if (loadThemePreference() === "system") applyThemePreference("system");
  };
  systemTheme.addEventListener("change", followSystemTheme);
  onDestroy(() => systemTheme.removeEventListener("change", followSystemTheme));

  loadAllLocales();
  const detectedLocale = detectLocale("en", ["en", "zh"], navigatorDetector);
  const savedLocale = loadLocalePreference();
  setLocale(
    savedLocale && isLocale(savedLocale) ? savedLocale : detectedLocale,
  );

  // No network activity on startup: checking for updates is the user's
  // choice (Settings → Check for Updates), matching the local-first ethos.
</script>

<div class="flex w-full flex-row h-full">
  <div id="nav" class="h-full flex flex-col z-20">
    <div>
      <ul class="menu bg-base-100 p-2">
        <li class="tooltip tooltip-right" data-tip={$LL.Nav.Notes()}>
          <a
            href={resolve("/notes")}
            class="flex justify-center items-center {$page.url.pathname ==
            '/notes'
              ? 'active'
              : ''}"
          >
            <Fa icon={faNoteSticky} size="1.4x" />
          </a>
        </li>
        <li class="tooltip tooltip-right my-1" data-tip={$LL.Nav.Sync()}>
          <a
            href={resolve("/sync")}
            class="flex justify-center items-center {$page.url.pathname ==
            '/sync'
              ? 'active'
              : ''}"
          >
            <Fa icon={faCopy} size="1.4x" />
          </a>
        </li>
      </ul>
    </div>
    <div class="flex-1"></div>
    <div>
      <ul class="menu bg-base-100 p-2">
        <li class="tooltip tooltip-right my-1" data-tip={$LL.Nav.About()}>
          <a
            href={resolve("/about")}
            class="flex justify-center items-center {$page.url.pathname ==
            '/about'
              ? 'active'
              : ''}"
          >
            <Fa icon={faCircleExclamation} size="1.4x" />
          </a>
        </li>
        <li class="tooltip tooltip-right" data-tip={$LL.Nav.Settings()}>
          <a
            href={resolve("/settings")}
            class="flex justify-center items-center {$page.url.pathname ==
            '/settings'
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
