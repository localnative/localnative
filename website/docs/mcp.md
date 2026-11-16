---
id: mcp
title: AI / MCP
---

LocalNative can give an AI assistant read-only access to your local notes and
archived article text through the [Model Context Protocol
(MCP)](https://modelcontextprotocol.io/). The server runs on your machine. It
does not include a model, send your library to a cloud service, or expose write,
delete, or sync commands.

## Install

Release bundles include `localnative` alongside the desktop app and the
permanent browser native-messaging host alias. Put the binary anywhere on your
machine and use its absolute path in the client configuration.

To install from a source checkout instead:

```bash
cargo install --path localnative-rs/localnative_cli --bin localnative --bin localnative-web-ext-host
```

The former `localnative-mcp` executable remains a compatibility alias in this
release. New configurations should use `localnative` with the `mcp` argument.

By default the server opens `~/LocalNative/localnative.sqlite3` on desktop.
`LOCALNATIVE_DB` selects another database, which is also useful for a backup or
a test library. The CLI also accepts `--db <FILE>` before or after `mcp`.

## Configure a client

Clients that use the common `mcpServers` JSON format can add:

```json
{
  "mcpServers": {
    "localnative": {
      "command": "/absolute/path/to/localnative",
      "args": ["mcp"],
      "env": {
        "LOCALNATIVE_DB": "/absolute/path/to/localnative.sqlite3"
      }
    }
  }
}
```

Omit the `env` object when using the default desktop database. Restart the
client after changing its MCP configuration.

The server provides two tools:

- `search_notes(query, limit, offset)` searches titles, URLs, tags,
  descriptions, and archived article text. Results contain metadata and archive
  availability, not full article bodies.
- `get_note(uuid4)` returns one result's complete note, archived text, and
  original source URL.

Ask the assistant to search first, open the relevant result, and cite its
source URL.

## Troubleshooting

- **The server does not start:** use the absolute binary path, keep
  `args: ["mcp"]`, and check that the file is executable. On macOS or Linux,
  run `chmod +x /absolute/path/to/localnative` if needed.
- **The library is empty:** confirm `LOCALNATIVE_DB` names the same SQLite file
  as the desktop app. Do not point it at a directory.
- **A bookmark has no article text:** only saves made with **save readable
  text** contain an archive. Re-save the page with that option enabled.
- **Search finds a note but the assistant cannot read it:** update both the
  core application and `localnative`; `get_note` requires the `get` command in
  the same core version.
- **Nothing leaves the machine by design:** MCP is stdio only. The AI client
  and any model it uses have their own privacy behavior; LocalNative does not
  control that separate connection.
