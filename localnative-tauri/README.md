# How to build

## Prerequisite
1. install node.js, npm (usually bundled with node.js) and yarn.
2. install tauri-cli by
    ```console
    $ cargo install tauri-cli
    ```

## Steps
> all commands run at root path of localnative-tauri

1. build frontend code
    ```console
    $ yarn run build
    ```
2. run with debug mode
    ```console
    $ cargo tauri dev
    ```
3. build with release mode
    ```console
    $ cargo tauri build
    ```
