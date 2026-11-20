#!/usr/bin/env python3
import json
import os
from pathlib import Path
import struct
import subprocess
import sys
import tempfile


def executable(directory, name):
    candidates = [name + ".exe", name + ".cmd", name] if os.name == "nt" else [name]
    for candidate in candidates:
        path = directory / candidate
        if path.is_file():
            return path
    raise AssertionError(f"missing installed command: {directory / name}")


def command(arguments):
    arguments = [str(argument) for argument in arguments]
    if os.name == "nt" and arguments[0].lower().endswith(".cmd"):
        return ["cmd.exe", "/d", "/s", "/c", subprocess.list2cmdline(arguments)]
    return arguments


def run(arguments, env, payload=None):
    result = subprocess.run(
        command(arguments),
        input=payload,
        capture_output=True,
        env=env,
        timeout=90,
    )
    assert result.returncode == 0, (
        arguments,
        result.returncode,
        result.stdout,
        result.stderr,
    )
    return result.stdout


def main():
    directory = Path(sys.argv[1]).resolve()
    cli = executable(directory, "localnative")
    host = executable(directory, "localnative-web-ext-host")
    version = run([cli, "--version"], os.environ.copy())
    print(version.decode().strip())

    with tempfile.TemporaryDirectory(prefix="localnative_package_check_") as temporary:
        home = Path(temporary)
        database = home / "library.sqlite3"
        env = os.environ.copy()
        env["HOME"] = str(home)
        env["USERPROFILE"] = str(home)
        env["LOCALNATIVE_DB"] = str(database)
        message = json.dumps(
            {
                "action": "insert",
                "title": "Package archive",
                "url": "https://example.com/package",
                "tags": "package",
                "description": "Installed package verification",
                "comments": "",
                "annotations": "packaged article body with a distinctive nebula passage",
                "limit": 10,
                "offset": 0,
                "is_public": False,
            }
        ).encode()
        process = subprocess.Popen(
            command([host, "chrome-extension://package-check/"]),
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            env=env,
        )
        stdout, stderr = process.communicate(
            struct.pack("@I", len(message)) + message,
            timeout=90,
        )
        assert process.returncode == 0, (process.returncode, stdout, stderr)
        length = struct.unpack("@I", stdout[:4])[0]
        inserted = json.loads(stdout[4 : 4 + length])
        uuid = inserted["notes"][0]["uuid4"]

        search = json.loads(run([cli, "search", "nebula", "--db", database], env))
        assert search["count"] == 1, search
        assert search["notes"][0]["annotations"] == "", search
        assert search["notes"][0]["has_archive"] is True, search
        note = json.loads(run([cli, "get", uuid, "--db", database], env))
        assert note["url"] == "https://example.com/package", note
        assert "nebula passage" in note["annotations"], note

        requests = [
            {"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}},
            {
                "jsonrpc": "2.0",
                "id": 2,
                "method": "tools/call",
                "params": {
                    "name": "search_notes",
                    "arguments": {"query": "nebula"},
                },
            },
            {
                "jsonrpc": "2.0",
                "id": 3,
                "method": "tools/call",
                "params": {"name": "get_note", "arguments": {"uuid4": uuid}},
            },
        ]
        payload = "".join(json.dumps(request) + "\n" for request in requests).encode()
        responses = [
            json.loads(line)
            for line in run([cli, "mcp", "--db", database], env, payload).splitlines()
        ]
        assert len(responses) == 3, responses
        search_text = responses[1]["result"]["content"][0]["text"]
        get_text = responses[2]["result"]["content"][0]["text"]
        assert uuid in search_text and "nebula passage" not in search_text
        assert "nebula passage" in get_text
        assert "https://example.com/package" in get_text

    print("package check passed: native host, search, get, and MCP")


if __name__ == "__main__":
    main()
