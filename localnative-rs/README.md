# localnative-rs

`cargo build`

# localnative-iced

### windows and mac

`cargo build`

### linux

Dependencies:
```bash
sudo apt install pkg-config libfreetype6-dev libfontconfig1-dev
```
and then `cargo build`

### Release
You can use the following command to package the program:
```bash
cargo xtask release
```
help:
```bash
xtask release
  Release iced and web-ext-host

  OPTIONS:
    --opengl
      Select opengl as the backend

    -t, --target <target>
      You can specify a target

    -p, --platform <platform>
      You can specify a platform, such as macos, windows, linux

    -v, --version <version>
      You can specify a version
```

If you do not set the version number, output the version according to the version of the xtask package.

If you set both the platform and the target, the target is preferred.

If you do not set the target and platform, compile with the currently running target.



## License
[AGPL-3.0](https://www.gnu.org/licenses/agpl-3.0.en.html)
