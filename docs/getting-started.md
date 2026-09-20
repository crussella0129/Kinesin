# Install and use Kinesin

Install the command, a llama.cpp runtime and a GGUF model once, then run
`kinesin` from any folder. Select a working folder and a local model with the
arrow keys and Enter. Kinesin starts its model server, checks readiness and
stops that server when the session ends. Normal local use needs no SSH login,
server URL or separately running terminal.

Choose the [Windows installation](#windows-powershell) or [Linux installation](#linux),
then [install a local model and runtime](#install-a-local-model-and-runtime) and
follow [your first session](#your-first-session). The
[explicit example workflow](#explicit-configurations-and-checked-tasks) is for
fixed profiles and automation; normal interactive use does not require a
`kinesin.toml` in each project.

## Windows PowerShell

### Prerequisites and checkout

If `cargo --version` and `git --version` already work, keep your existing tools.
Otherwise install Git for Windows and Rust through the official
[rustup installer](https://rustup.rs/). Use the MSVC Rust target. Install Visual
Studio Build Tools with **Desktop development with C++**, including the MSVC
compiler and Windows SDK. The native TLS dependency also needs NASM available to
the build; see its [Windows requirements](https://aws.github.io/aws-lc-rs/requirements/windows.html).
Reopen PowerShell after installation. A Developer PowerShell for Visual Studio
can provide the compiler environment if ordinary PowerShell cannot find it.

For a new checkout:

```powershell
cd $HOME
git clone --branch dev https://github.com/crussella0129/Kinesin.git Kinesin
cd .\Kinesin
```

For your existing checkout, use just:

```powershell
cd "$HOME\Kinesin"
```

From that directory, install the pinned toolchain and confirm the CLI starts:

```powershell
rustup toolchain install 1.96.0 --profile minimal --component rustfmt,clippy
cargo run --locked -- --help
```

This builds and starts **kinesin**, then prints usage and exits successfully.
Help does not need a configuration or model. The `--` separates Cargo options
from Kinesin options. The first build can take several minutes.

### Install the command

From the checkout:

```powershell
cargo install --locked --path . --bin kinesin
$env:Path = "$HOME\.cargo\bin;$env:Path"
Get-Command kinesin
kinesin --help
```

Installation builds an optimized binary at
`C:\Users\YOUR_NAME\.cargo\bin\kinesin.exe` by default. For a faster development
installation that reuses your debug build, append `--debug` to the install
command. Neither fixture executable is installed.

Rustup normally puts this directory on PATH. The assignment above updates the
current PowerShell session. If a new terminal still cannot find Kinesin, add
`%USERPROFILE%\.cargo\bin` through **Edit environment variables for your account
→ Path → New**, then reopen PowerShell. These paths assume an unmodified
`CARGO_HOME`; a custom Cargo install root has its own `bin` directory.

After installation, you can leave the checkout:

```powershell
cd $HOME
kinesin
```

Before the first session, complete the [model/runtime setup](#install-a-local-model-and-runtime).
You do not need to run Cargo or copy configuration into each project. During
development, `cargo run --locked` in the checkout opens the same terminal entry.

## Linux

### Prerequisites and checkout

On Debian/Ubuntu, the source build needs a C/C++ compiler, Make, Git, curl and
CA certificates. Install these if missing (this step needs an administrator):

```bash
sudo apt-get update
sudo apt-get install build-essential git curl ca-certificates
```

The current non-FIPS native TLS build uses the C compiler. See the
[Linux native requirements](https://aws.github.io/aws-lc-rs/requirements/linux.html)
if building another target or dependency configuration. This is not a request
to install GPU drivers; model serving has its own runtime requirements.

If Rust is not installed, use the official installer and load its PATH into the
current shell:

```bash
(
    set -e
    rustup_installer="$(mktemp)"
    trap 'rm -f -- "$rustup_installer"' EXIT
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -o "$rustup_installer"
    sh "$rustup_installer" --profile minimal
)
. "$HOME/.cargo/env"
```

Then use a new checkout:

```bash
cd "$HOME"
git clone --branch dev https://github.com/crussella0129/Kinesin.git Kinesin
cd Kinesin
rustup toolchain install 1.96.0 --profile minimal --component rustfmt,clippy
cargo run --locked -- --help
```

With an existing checkout, start at `cd "$HOME/Kinesin"` and omit `git clone`.
The repository's `rust-toolchain.toml` selects 1.96.0. For a memory-constrained
build, prefix a Cargo command with `CARGO_BUILD_JOBS=2`.

### Install the command

From the checkout:

```bash
cargo install --locked --path . --bin kinesin
export PATH="$HOME/.cargo/bin:$PATH"
command -v kinesin
kinesin --help
cd "$HOME"
kinesin
```

The default binary is `$HOME/.cargo/bin/kinesin`. Source `$HOME/.cargo/env`
or add that directory to your shell's PATH for future sessions if necessary.
Custom `CARGO_HOME` or installation roots change the binary location.

Append `--debug` for a faster development install that reuses the debug build.
Use a native Linux filesystem for builds. `CARGO_BUILD_JOBS=2` can reduce build
memory pressure on smaller machines. The same installed command opens the folder
selector on Debian; no Windows path or checkout-local configuration is needed.

## Install a local model and runtime

Kinesin is the assistant/controller. **llama-server** is the llama.cpp program
that loads a **GGUF** model and performs inference. They are separate executables
because inference is supplied by llama.cpp. Kinesin manages that process for a
normal session; the two test fixtures in the repository are unrelated.

Choose a trusted llama.cpp runtime build for your operating system and hardware,
and a GGUF model that supports the chat/tool protocol you need. The Git checkout
does not contain either download. A working GPU driver is required for GPU
acceleration. Installing Kinesin does not install a driver or download weights.
The [verified example downloads](#verified-example-downloads) below give one
previously tested combination; other GGUF models are not automatically equivalent.

Use these per-user locations so the model remains available from any launch folder:

| Platform | GGUF files | Runtime executable and its companion libraries |
| --- | --- | --- |
| Windows | `%LOCALAPPDATA%\Kinesin\models\model-example.gguf` | `%LOCALAPPDATA%\Kinesin\runtime\llama-server.exe` |
| Linux | `$XDG_DATA_HOME/kinesin/models/model-example.gguf`, default `~/.local/share/kinesin/models/model-example.gguf` | `$XDG_DATA_HOME/kinesin/runtime/llama-server`, default `~/.local/share/kinesin/runtime/llama-server` |

Alternatively, use a **portable layout beside the installed Kinesin executable**:

```text
Kinesin/
  kinesin.exe
  models/
    model-example.gguf
  runtime/
    llama-server.exe
    ... companion runtime libraries ...
```

On Linux, the executable names are `kinesin` and `llama-server`. Keep all runtime
dependencies with the backend as supplied by its distribution. This layout also
works when Kinesin is installed in Cargo's `bin` directory: put `models/` and
`runtime/` inside that same `bin` directory. It works from other launch folders
and does not require models under AppData or an XDG data directory. Use a trusted
installation directory outside your working folders. Personal settings and
history still use the [per-user locations](#where-settings-live).

Create the directories on **Windows PowerShell**:

```powershell
$kinesinData = Join-Path $env:LOCALAPPDATA 'Kinesin'
New-Item -ItemType Directory -Force -Path "$kinesinData\models", "$kinesinData\runtime" | Out-Null
```

Extract your downloaded runtime archive. Copy the runtime distribution into
`$kinesinData\runtime` so `llama-server.exe` is directly in that directory. Keep
its DLLs and other runtime dependencies alongside it as supplied by the archive;
copying only the executable can prevent startup. Copy your downloaded GGUF into
`$kinesinData\models`. Its actual filename can be anything ending in `.gguf`.
Confirm the locations:

```powershell
Get-Item -LiteralPath "$kinesinData\runtime\llama-server.exe"
Get-ChildItem -LiteralPath "$kinesinData\models" -Filter '*.gguf'
kinesin
```

Create the directories on **Linux**:

```bash
kinesin_data="${XDG_DATA_HOME:-$HOME/.local/share}/kinesin"
mkdir -p "$kinesin_data/models" "$kinesin_data/runtime"
```

Extract the Linux runtime archive and copy its distribution into
`$kinesin_data/runtime`, preserving its libraries and subdirectories, with
`llama-server` directly in `runtime`. Copy your GGUF into `$kinesin_data/models`.
Ensure the runtime is executable and confirm both locations:

```bash
chmod u+x "$kinesin_data/runtime/llama-server"
test -x "$kinesin_data/runtime/llama-server"
ls "$kinesin_data/models/"*.gguf
kinesin
```

If you already keep these files elsewhere, leave them there and select their
paths with `--model-path` and `--runtime-path` as described below. Kinesin also
checks trusted absolute PATH entries for `llama-server` when it is not installed
in the per-user runtime directory. It does not execute a runtime merely because
one appears in the selected project folder.

## Your first session

Run:

```text
kinesin
```

The welcome screen asks for a **working folder**, displaying the current
directory as the default. Press Enter to use it or enter another existing
project-folder path. Relative paths resolve from where you launched Kinesin;
paths containing spaces are supported. Invalid selections are explained and
asked again. Select a project folder rather than your entire home directory:
Kinesin keeps private settings/history under your home and does not expose them
as workspace files.

Next, the **model selector** lists discovered GGUF files. Move the caret with
Up/Down and press Enter to select. The previous selection is preferred on later
launches, so Enter reuses it. Choose **Choose another GGUF file...** to enter a
file path outside the discovered folders. Missing files or runtimes produce
setup instructions before a task can run.

Discovery checks direct files in the per-user models directory, plus `model/`
and `models/` in the launch directory and beside the Kinesin executable. It is a
bounded, nonrecursive scan, not a search of your entire disk. An explicitly
selected GGUF path is saved and remains available when you launch elsewhere.

The GGUF's containing directory and runtime directory must be separate from the
working folder: neither may contain or be contained by it. If you select a model
from a project's `models/` directory, Kinesin asks for a disjoint working folder.
Using the per-user models directory avoids this conflict. Selecting a model or
runtime gives the backend access to that input; it does not give the assistant
file tools access to its containing directory.

Kinesin starts the selected local runtime and waits for verified readiness before
accepting a request. You do not need to start `llama-server` yourself. On exit or
Ctrl+C, Kinesin stops the server process tree that it started.

To skip the model chooser, supply a path. For example, on Windows:

```powershell
kinesin --model-path 'D:\Models\model-example.gguf' --runtime-path 'D:\llama-runtime\llama-server.exe'
```

On Linux:

```bash
kinesin --model-path "$HOME/models/model-example.gguf" --runtime-path "$HOME/llama-runtime/llama-server"
```

Replace the example paths with your existing files. `--runtime-path` is optional
when the runtime is discoverable. Paths containing spaces must be quoted at the
shell; relative paths resolve from the launch directory. Working-folder selection
still happens when these flags are supplied.

The session shows the actual folder, model and permitted actions. The personal
profile initially allows listing, reading, searching, creating folders, writing
files and editing files. It does not grant arbitrary command execution. File
paths in your requests refer to the selected folder.

For example:

```text
> Remember that this project is called Lantern.
> Create project.txt with that project name.
> Edit that file to add status: draft.
```

You see tool activity and the model's readable answer. File creation and editing
must be real tool operations inside the chosen folder; the model's claim alone
is not verification. You can also ask to create folders without a command shell.
An existing folder is reported without replacement, and parent folders must
already exist.

| Session command | Use |
| --- | --- |
| `/help` | Show available session commands |
| `/status` | Inspect the latest run and its acceptance status |
| `/permissions` | Show the configured tools and command grants |
| `/context` | Show recent memory usage and counts of shortened or omitted turns |
| `/new` or `/clear` | Clear all recent conversation memory |
| `/exit` | Leave the session |

Ctrl+C cancels and stops the session. Each request is journaled as an immutable
run. Follow-ups remember recent completed prompts and answers during this process;
failed entries are excluded while earlier completed turns remain within the
memory limits. Large text is shortened and older turns are removed as memory
fills, with visible notices.
`/context` reports the limits and adjustments; [the CLI reference](cli.md#the-session)
describes the exact byte and turn bounds. Exiting discards this memory, so a later
launch starts fresh. File tools re-read current contents instead of relying on
retained tool results. A freeform result has no independent acceptance checker;
that is shown under `/status` and is distinct from an execution failure.

Each request uses the configured permissions anew. The default metadata journal
retains run results but excludes the recent conversation transcript; retaining
private inputs for replay requires explicitly selected replay capture.

### Where settings live

| Platform | Personal settings | Private journal |
| --- | --- | --- |
| Windows | `%APPDATA%\Kinesin\settings.toml` | `%LOCALAPPDATA%\Kinesin\state\kinesin.sqlite` |
| Linux | `$XDG_CONFIG_HOME/kinesin/settings.toml`, default `~/.config/kinesin/settings.toml` | `$XDG_STATE_HOME/kinesin/kinesin.sqlite`, default `~/.local/state/kinesin/kinesin.sqlite` |

Setup creates new private settings/state directories and preserves existing
permissions and tool grants. The working folder applies to the current session;
the selected local model/runtime are saved for later launches. Model weights and
runtime files stay where you installed them. Edit the personal TOML to change
tool grants or advanced runtime settings.

### Explicit and machine entry

`kinesin --config PATH` uses that profile's configured workspace and skips the
folder selector. It does not silently enlarge that profile's permissions.

`kinesin --json --config PATH` emits session JSON Lines for automation.
Piped input skips setup prompts and uses an explicit configuration or the
current directory's `kinesin.toml`; it does not unexpectedly enter a setup wizard.
One-shot, batch, inspect/export/replay and service commands keep their structured
interfaces. There is no `kinesin run` subcommand.

`--model-path`, `--runtime-path` and `--external` are for human terminal sessions.
They cannot be combined with `--config`, `--json`, piped input or a one-shot
task/prompt. `--external` also cannot be combined with the local path flags.
Human setup requires both stdin and stdout to be terminals. If you redirect
output, use an explicit profile, for example
`kinesin --config PATH --json > session.jsonl`; redirected output does not open
the interactive setup or model selector.

## Preview a local website

Website previews require an explicit local configuration with one workspace
and one model. Keep its configuration and private state outside the working
folder. In that workspace's existing entry, include `start_preview` alongside
the file tools you want to grant, for example:

```toml
tools = ["list_files", "read_file", "search_files", "create_directory", "write_file", "edit_file", "start_preview"]
```

Use a local freeform profile. Service configurations reject preview grants,
and checked runs cannot use a workspace that grants previews. The ordinary
personal setup does not add the grant automatically. Start the interactive profile:

```text
kinesin --config preview.toml
```

Ask Kinesin to create a small storefront with `index.html`, `styles.css` and
`app.js`, then call `start_preview` for its directory. Use relative asset paths
such as `./styles.css` and `./app.js`, and JavaScript `addEventListener` calls.
Inline scripts, inline styles, event attributes and remote assets are blocked.
The returned URL includes a private path prefix; open the complete URL.

Keep this Kinesin session open while using the website. Further file edits
appear after browser reload, and requesting the same preview again returns
the same URL. `/exit`, EOF or Ctrl+C closes the server; a one-shot `--prompt`
command closes it when that command finishes. Restart the session to preview
a different directory.

This server serves static HTML, CSS, JavaScript, JSON, image and font files up
to 1 MiB each, with no directory listings or backend execution. See the
[preview tool contract](loop-and-tools.md#local-website-previews) for its
complete limits.

## External model servers (advanced)

Use an external connection only when you already run a compatible server yourself
or intentionally place inference on another computer:

```text
kinesin --external
```

Choose the working folder. First external setup, or switching from a saved local
model, asks for the **model server URL** and **served model ID**. Later external
sessions reuse the saved connection; edit its URL/ID in your personal settings
when changing that server. The served ID is its API identity, usually the value passed
to llama-server's `--alias`. Kinesin checks the connection but does not start or
stop this externally managed process. The URL and served ID belong to this
explicit external profile; bare `kinesin` uses local GGUF selection.

For a manually started local server, use its loopback URL, such as
`http://127.0.0.1:8080`. Explicit TOML profiles and automation also attach to the
endpoint configured in their model profile. `--model local` in those commands
selects the **configuration alias**, not a GGUF file or an inference machine.

### Optional remote-server connection

Use this section only when you intentionally run the model server on another
computer. It requires access to that computer through SSH; a local model server
does not need SSH, a remote username, or a tailnet. Replace `model-user`,
`model-host`, and the runtime/model paths below with your own values.

On the client computer, open a separate terminal:

```powershell
ssh model-user@model-host
```

In the remote Linux shell, start the runtime and model you installed there.
This example uses an explicit server alias and one 4096-token slot:

```bash
cd "$HOME/llama-runtime"
./llama-server -m "$HOME/models/model-example.gguf" --host 127.0.0.1 --port 18080 --alias model-example -c 4096 -np 1 --jinja --no-context-shift -ngl 99 -t 4 -tb 4 --no-webui
```

Leave it running. In a **second client terminal**, open the authenticated local
forward and leave that running too:

```powershell
ssh -N -T -o ExitOnForwardFailure=yes -L 127.0.0.1:8080:127.0.0.1:18080 model-user@model-host
```

In a **third client terminal**, test the forwarded endpoint. On PowerShell:

```powershell
Invoke-RestMethod http://127.0.0.1:8080/health
```

On Linux, use `curl --fail http://127.0.0.1:8080/health`. Wait for `status: ok`,
then run `kinesin --external` in this third terminal. Enter
`http://127.0.0.1:8080` and `model-example`. Keep the server and tunnel terminals
open. Stop each with Ctrl+C when finished. SSH access and forwarding must be
enabled on the remote machine. A direct `http://model-host:18080` URL is neither
the intended listener nor accepted by the non-loopback HTTPS policy.

SSH may ask you to confirm a new host key, unlock a private key with its
passphrase, or authenticate with the remote account's password. These are
standard SSH prompts: verify a new host fingerprint through a trusted channel,
then answer in that terminal. Kinesin does not create an SSH account, log in
automatically, or collect/store SSH passwords. These foreground commands retain
the prompts rather than using a hidden background login or `BatchMode=yes`.
See the [OpenSSH manual](https://man.openbsd.org/ssh).

If Kinesin and your manually managed server run on the same computer, skip SSH
entirely. Run `kinesin --external` and enter that server's actual loopback port
and served ID. A normal local GGUF session does not need any of this setup.

## Verified example downloads

The following **historical verified example** uses llama.cpp b6500 and
Qwen2.5-Coder-7B-Instruct Q4_K_M. It is an optional compatibility baseline, not a
required model or a claim that all runtime/model versions behave alike. The
public model filename below is an artifact name, not a personal machine path.

Download and extract the appropriate official
[Windows Vulkan ZIP](https://github.com/ggml-org/llama.cpp/releases/download/b6500/llama-b6500-bin-win-vulkan-x64.zip)
or [Linux Vulkan ZIP](https://github.com/ggml-org/llama.cpp/releases/download/b6500/llama-b6500-bin-ubuntu-vulkan-x64.zip).
Download the pinned
[Qwen GGUF](https://huggingface.co/Qwen/Qwen2.5-Coder-7B-Instruct-GGUF/resolve/13fb94bfda8c8cf22497dc57b78f391a9acb426a/qwen2.5-coder-7b-instruct-q4_k_m.gguf)
(about 4.7 GB) separately. The Vulkan runtime needs a working driver for the
selected GPU; this guide does not install that driver.

Check the downloads against these SHA-256 values (`Get-FileHash -Algorithm SHA256
PATH` on Windows, `sha256sum PATH` on Linux):

| Download | SHA-256 |
| --- | --- |
| Windows ZIP | `d485ce9cdda9967d2b27a054bb7e16b57a56a332a4b54b2f02964b95ee591548` |
| Linux ZIP | `ee15538d98808f15d13a474d2554bc708a475e35bf560e1d0d21b9e96bc5f8ea` |
| GGUF | `509287f78cb4d4cf6b3843734733b914b2c158e43e22a7f4bf5e963800894d3c` |

For normal use, place these files in the [per-user locations](#install-a-local-model-and-runtime)
and run `kinesin`; the application starts the server. To use the
**explicit configuration** examples instead, keep a manually started server on
port 8080 with their required alias. On PowerShell, a quoted executable path
requires `&`:

```powershell
& 'C:\path\to\llama-server.exe' -m 'C:\path\to\qwen2.5-coder-7b-instruct-q4_k_m.gguf' --host 127.0.0.1 --port 8080 --alias model-example -c 4096 -np 1 --jinja --no-context-shift -ngl 99 -t 4 -tb 4 --no-webui
```

On Linux, invoke your `llama-server` path with the same flags and replace the
GGUF path. Leave that manually started server running for the explicit commands
below. See [model preflight](model-preflight.md) for
recorded exchanges and [deployment evidence](sprints/s10/sprint-tests/remote-deployment.md)
for the tested GPU allocation. Direct non-loopback model URLs require HTTPS;
an authenticated SSH forward supplies encryption between loopback endpoints.

## Explicit configurations and checked tasks

The tracked `kinesin.example.toml` is intentionally a **read-only example** for
the `practice-fields` checker. It is not the normal interactive profile.
Use it when you want fixed configuration, JSON automation or the following
checked-task example. Keep its private state and configuration outside its
workspace. An explicit profile must grant `create_directory` to create folders;
read/list permission alone cannot do so.

### Windows example setup

Run this block from the repository root. It copies the starter configuration
and creates the state/workspace only when missing. Existing files and directory
permissions are preserved; review an existing `kinesin.toml` instead of assuming
it contains the starter settings.

```powershell
if (-not (Test-Path -LiteralPath '.\state')) {
    New-Item -ItemType Directory -Path '.\state' | Out-Null
    $operatorSid = [Security.Principal.WindowsIdentity]::GetCurrent().User.Value
    icacls '.\state' /inheritance:r /grant:r "*${operatorSid}:(OI)(CI)F" '*S-1-5-18:(OI)(CI)F' '*S-1-5-32-544:(OI)(CI)F' | Out-Null
    if ($LASTEXITCODE -ne 0) { throw 'Could not restrict the new state directory.' }
}
if (-not (Test-Path -LiteralPath '.\workspace')) {
    New-Item -ItemType Directory -Path '.\workspace' | Out-Null
}
if (-not (Test-Path -LiteralPath '.\kinesin.toml')) {
    Copy-Item -LiteralPath '.\kinesin.example.toml' -Destination '.\kinesin.toml'
}
if (-not (Test-Path -LiteralPath '.\workspace\project.txt')) {
    @('project=Kinesin', 'language=Rust') | Set-Content -LiteralPath '.\workspace\project.txt' -Encoding ascii
}
```

The new state directory grants access to your account, SYSTEM and Administrators.
The CLI does not automatically repair or audit existing ACLs. Keep configuration
and state outside the tool workspace. This starter grants only list/read tools.

### Linux example setup

Run this from the checkout. The subshell applies private permissions to newly
created items without changing your shell's umask. Repeating it preserves
existing configuration, data and directory permissions.

```bash
(
    umask 077
    mkdir -p state workspace
    if [ ! -e kinesin.toml ]; then cp kinesin.example.toml kinesin.toml; fi
    if [ ! -e workspace/project.txt ]; then
        printf 'project=Kinesin\nlanguage=Rust\n' > workspace/project.txt
    fi
)
```

New directories are mode 700 and new files mode 600. Inspect existing state
permissions separately; the CLI does not change them for you. Configuration
and state remain outside the tool workspace.

### One-shot example

After the explicit example setup and model connection:

```text
kinesin --config kinesin.toml --workspace practice --model local --prompt "Say hello" --allow-unchecked
```

This command emits a JSON line. `phase: completed` and
`acceptance_status: unchecked` mean execution finished without an independent
checker; `task_accepted: false` does not by itself mean the operation failed.
The answer is in `result.candidate`. `--allow-unchecked` permits exit 0 for that
freeform result. For the same explicit profile with readable interaction:

```text
kinesin --config kinesin.toml
```

### Check a file task and replay it

The starter contains the `practice-fields` task, which checks the project and
language values against the actual `workspace/project.txt`. Request replay
capture when creating the run:

```text
cargo run --locked -- --task practice-fields --model local --capture replay
```

A successful checked run reports `acceptance_status: "passed"`,
`task_accepted: true`, and exit 0. If the model fails to produce a supported,
evidenced answer, the receipt reports that failure; text alone is not proof.
Copy the actual `run_id` from its JSON output. Run these one at a time,
substituting that ID and choosing a new export filename:

```text
cargo run --locked -- inspect --config kinesin.toml --run RUN_ID
cargo run --locked -- export --config kinesin.toml --run RUN_ID --output state/first-run.json
cargo run --locked -- replay --input state/first-run.json
```

Stop any other Kinesin session using this state before running operator commands.
Export never overwrites an existing file. Replay needs no model server and
reports whether recorded decisions are consistent; replay success is separate
from the original task's acceptance. The starter defaults to metadata capture,
which cannot support exact replay unless you selected `--capture replay` for
the original run. Replay files contain private run content; keep them in private
storage. See [CLI reference](cli.md) for batch, service and retention commands.

## Why Cargo lists three binaries

| Binary | Purpose | Install/use for normal work? |
| --- | --- | --- |
| `kinesin` | The actual CLI, interactive session and optional service | Yes |
| `cmd-fixture` | A real child process used to test argv, output limits and process cleanup | No |
| `mcp-fixture` | A small stdio server used to test MCP protocol and lifecycle behavior | No |

The tests need real separate processes to exercise process termination and stdio
boundaries. They are not extra services you must start, and are not model servers.
`default-run = "kinesin"` selects the product for `cargo run`.
`cargo install --locked --path . --bin kinesin` installs just that executable.
See [Cargo's install rules](https://doc.rust-lang.org/cargo/commands/cargo-install.html).

## Troubleshooting

| Symptom | Meaning and fix |
| --- | --- |
| `kinesin` is not recognized / command not found | Install with `--bin kinesin` and put Cargo's `bin` directory on PATH. `cargo run` only runs a checkout build. |
| Cargo cannot choose a binary | Update your checkout, or use `cargo run --bin kinesin`. Current Cargo defaults select the product. |
| Cannot open an explicit configuration | Check the displayed path; create the example only for an explicit workflow, or start bare `kinesin` in a terminal for personal setup. |
| Only `project.txt` appears, or writes are disabled | You selected the read-only example. Start bare `kinesin` for the personal folder selector, or edit the explicit profile's root and tool grants. |
| Folder overlaps settings, history, model or runtime | Choose a disjoint working folder; keep GGUF/runtime files in the per-user data directory. Selecting these inputs does not expose their directories to assistant tools. |
| Folder already exists | Directory creation does not replace existing entries. Pick another name or use the existing directory. |
| No GGUF files found | Put an actual GGUF in the per-user models directory, select another file in the chooser, or use `--model-path PATH`. Discovery does not scan subdirectories. |
| Cannot find/start llama-server | Install the runtime and its libraries in the per-user runtime directory, or use `--runtime-path PATH`. Check executable permissions on Linux and the runtime's driver requirements. |
| Local model fails readiness or exits | Read the displayed runtime diagnostic. Check the model/runtime combination, available RAM/VRAM and required libraries/driver; Kinesin stops its failed child before returning. |
| External model connection failure | Start your external server and any intentionally configured SSH forward; check `/health`, served ID, context and slots. Use `--external` only for that workflow. |
| SSH asks for a password or host key | This belongs to the optional manual remote connection. Answer the SSH prompt in its terminal after checking the host identity. Ordinary `kinesin` with a local GGUF does not use SSH. |
| Explicit session has multiple models/workspaces | Use a profile with one of each or a one-shot command selecting aliases. |
| Unexpected JSON | Remove `--json` for human sessions. One-shot/operator commands intentionally remain structured. |
| State is locked | Exit the other Kinesin session using that same state. |
| Replay capture is unavailable | Select `--capture replay` when creating the original run; export cannot invent missing inputs. |
| Compiler/linker or disk-full error | Resolve build prerequisites or disk capacity, then rerun the installation command. |
