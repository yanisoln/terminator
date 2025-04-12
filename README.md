# terminator 🤖

[![docs](https://img.shields.io/badge/read_the-docs-blue)](https://docs.screenpi.pe/terminator/introduction)
[![discord](https://img.shields.io/discord/1158344578124554270?label=discord)](https://discord.gg/dU9EBuw7Uq)

high level demo:

<https://github.com/user-attachments/assets/024c06fa-19f2-4fc9-b52d-329768ee52d0>

dev demo 1:

<https://github.com/user-attachments/assets/890d6842-782c-4b2b-8920-224bd63c4545>

dev demo 2:

<https://github.com/user-attachments/assets/c9f472f7-79ed-49c6-a4d0-93608fa1ce55>

**terminator** is an ai-first cross-platform ui automation library for rust, designed to interact with native gui applications on windows and macos using a playwright-like api.

it provides a unified api to find and control ui elements like buttons, text fields, windows, and more. because it uses os-level accessibility apis, it is **100x faster and more reliable** for ai agents than vision-based approaches.

> **Note:** while we support macos and windows, we are currently focusing development efforts on windows.

## documentation

for detailed information on features, installation, usage, and the api, please visit the **[official documentation](https://docs.screenpi.pe/terminator/introduction)**.

## quick start

1.  **clone the repo:**
    ```bash
    git clone https://github.com/mediar-ai/terminator
    cd terminator
    ```
2.  **download & unzip the server (windows cli):**
    ```powershell
    .\setup_windows.ps1
    ```
3.  **run the server:**
    ```powershell
    ./server_release/server.exe --debug
    ```
4.  **run an example client (in another terminal):**
    ```bash
    # make sure node.js/bun is installed
    cd examples # navigate back to the examples directory in the cloned repo
    npm i # or bun/yarn
    npx tsx client_example.ts
    # or python:
    python client_example.py
    ```

*check the [getting started guide](https://docs.screenpi.pe/terminator/getting-started) in the docs for more details.*

## key dependencies

*   windows: [uiautomation-rs](https://github.com/leexgone/uiautomation-rs)
*   macos: [macos accessibility api](https://developer.apple.com/documentation/appkit/nsaccessibility) (considering switch to [cidre](https://github.com/yury/cidre))
*   debugging: [accessibility insights for windows](https://accessibilityinsights.io/downloads/)

## contributing

contributions are welcome! please feel free to submit issues and pull requests. many parts are experimental, and help is appreciated. join our [discord](https://discord.gg/dU9EBuw7Uq) to discuss.