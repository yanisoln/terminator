# terminator 🤖

<p style="text-align: center;">
    <a href="https://discord.gg/dU9EBuw7Uq">
        <img src="https://img.shields.io/discord/823813159592001537?color=5865F2&logo=discord&logoColor=white&style=flat-square" alt="Join us on Discord">
    </a>
    <a href="https://docs.screenpi.pe/terminator/introduction">
        <img src="https://img.shields.io/badge/read_the-docs-blue" alt="docs">
    </a>
</p>

### videos 

- [📹 Enterprise PDF to windows legacy app form](https://github.com/user-attachments/assets/024c06fa-19f2-4fc9-b52d-329768ee52d0)
- [📹 Technical overview video](https://youtu.be/ycS9G_jpl04)
- [📹 Technical overview PDF to windows legacy app form](https://www.youtube.com/watch?v=CMw3iexyCMI)

**terminator** is an ai-first cross-platform ui automation library for rust, designed to interact with native gui applications on windows and macos using a playwright-like api.

it provides a unified api to find and control ui elements like buttons, text fields, windows, and more. because it uses os-level accessibility apis, it is **100x faster and more reliable** for ai agents than vision-based approaches.

`terminator` can also parse and interact with background apps/windows, unlike vision based approach.

> **⚠️ Note 1 ⚠️:** while we support macos and windows, we are currently focusing development efforts on windows, so you will have to figure out yourself how to use the macos version.

> **⚠️ Note 2 ⚠️:** keep in mind it's very experimental, expect many bugs, report it and we'll fix in minutes.

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
    powershell -ExecutionPolicy Bypass -File .\setup_windows.ps1
    ```
3.  **run the server:**
    ```powershell
    ./server_release/server.exe --debug
    ```
4.  **run an example client (in another terminal):**
    ```bash
    cd examples/hello-world
    npm i
    npm run dev
    # open http://localhost:3000
    ```

*check the [getting started guide](https://docs.screenpi.pe/terminator/getting-started) in the docs for more details.*

*check the [Vercel's AI SDK tool call example](https://github.com/mediar-ai/terminator/tree/main/examples/pdf-to-form) to use AI with `terminator`.*

*check [how to Vibe Work using MCP](https://github.com/mediar-ai/terminator/tree/main/mcp).*


## key dependencies

*   windows: [uiautomation-rs](https://github.com/leexgone/uiautomation-rs)
*   macos: [macos accessibility api](https://developer.apple.com/documentation/appkit/nsaccessibility) (considering switch to [cidre](https://github.com/yury/cidre))
*   debugging: [accessibility insights for windows](https://accessibilityinsights.io/downloads/)

## contributing

contributions are welcome! please feel free to submit issues and pull requests. many parts are experimental, and help is appreciated. join our [discord](https://discord.gg/dU9EBuw7Uq) to discuss.

## businesses 

if you want desktop automation at scale for your business, [let's talk](https://screenpi.pe/enterprise)
