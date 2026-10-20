import { invoke } from "@tauri-apps/api/core";
import { emit } from "@tauri-apps/api/event";
import { debounce } from "underscore";

export function cmdSearchOrFilter(searchText: string) {
  const range = globalThis.AppState.range;
  if (range) {
    cmdFilter(searchText, range[0], range[1]);
  } else {
    // Paging keeps the query — the old `select` here silently dropped it.
    const message = {
      action: "search",
      query: searchText,
      limit: globalThis.AppState.limit,
      offset: globalThis.AppState.offset,
    };

    cmd(message);
  }
}

/** Notes created between `from` and `to` (inclusive, `YYYY-MM-DD`). */
export const cmdFilter = debounce((searchText: string, from: string, to: string) => {
  const message = {
    action: "filter",
    query: searchText,
    limit: globalThis.AppState.limit,
    offset: globalThis.AppState.offset,
    from: from,
    to: to,
  };
  cmd(message);
}, 500);

export function cmdDelete(searchText: string, rowid: number) {
  const message = {
    action: "delete",
    query: searchText,
    rowid: rowid,
    limit: globalThis.AppState.limit,
    offset: globalThis.AppState.offset,
  };

  cmd(message);
}

export function cmdSearchImp(searchText: string) {
  globalThis.AppState.clearOffset();
  globalThis.AppState.range = null;
  globalThis.AppState.query = searchText;

  const message = {
    action: "search",

    query: searchText,
    limit: globalThis.AppState.limit,
    offset: globalThis.AppState.offset,
  };
  cmd(message);
}

export function cmdSelect() {
  globalThis.AppState.clearOffset();
  globalThis.AppState.range = null;
  globalThis.AppState.query = "";

  const message = {
    action: "select",
    limit: globalThis.AppState.limit,
    offset: globalThis.AppState.offset,
  };
  cmd(message);
}

export function cmdSyncViaAttach(uri: string) {
  const message = {
    action: "sync-via-attach",
    uri: uri,
  };
  cmd(message);
}

/** Start the sync server (no new pairings). */
export function cmdServer() {
  cmd({ action: "server", addr: "0.0.0.0:2345" });
}

/** Start the server (if needed) and accept one new pairing. */
export function cmdServerPairing() {
  cmd({ action: "server-pairing", addr: "0.0.0.0:2345" });
}

/** Stop the sync server running in this process. */
export function cmdServerStop() {
  cmd({ action: "server-stop", addr: "0.0.0.0:2345" });
}

/** Sync with a peer; `code` pairs on first contact. */
export function cmdClientSync(addr: string, code?: string) {
  const message: { action: string; addr: string; code?: string } = {
    action: "client-sync",
    addr: addr,
  };
  if (code) {
    message.code = code;
  }
  cmd(message);
}

function cmd(message: any) {
  const input = JSON.stringify(message, null, 2);

  invoke<string>("input", { input })
    .then((res) => {
      let resp: any;
      try {
        resp = JSON.parse(res);
      } catch {
        emit("syncStatus", { error: res });
        return;
      }

      // Bare-string replies (e.g. "Sync via attach completed") are statuses.
      if (typeof resp !== "object" || resp === null) {
        emit("syncStatus", { ok: String(resp) });
        return;
      }

      // Every core failure arrives as {"error": <message>, "code": <code>}.
      if (resp.error) {
        console.warn("core error:", resp.code, resp.error);
        emit("syncStatus", { error: resp.error, code: resp.code });
        return;
      }

      // Sync/server results reach the sync page.
      if (resp["client-sync"]) {
        emit("syncStatus", { ok: resp["client-sync"] });
        return;
      }
      if (resp.server !== undefined) {
        emit("syncStatus", {
          server: resp.server,
          addresses: resp.addresses,
          pairingCode: resp["pairing-code"],
        });
        return;
      }

      if (resp.count !== undefined) {
        globalThis.AppState.count = resp.count;
      }

      onNativeMessage(resp);
    })
    .catch((err) => {
      console.warn("invoke failed:", err);
      emit("syncStatus", { error: String(err) });
    });
}

// A query result as the core returns it (`QueryResult` in the core's models).
function onNativeMessage(message: {
  days: Array<{ date: string; count: number }>;
  notes: any;
  tags: Array<{ tag: string; count: number }>;
  count: number;
}) {
  emit("refreshTags", { tags: message.tags });

  if (!message.notes) return;

  // update pagination text
  if (message.count >= 0) {
    emit("refreshPaginationText", {
      pagination_text: globalThis.AppState.makePaginationText(),
    });
  }

  emit("refreshNotes", { notes: message.notes });

  // The histogram spans every day whatever the page or range, so only the
  // first page of a query redraws it; paging leaves the zoom alone.
  if (message.days && globalThis.AppState.offset == 0) {
    emit("refreshChart", { days: message.days });
  }
}
