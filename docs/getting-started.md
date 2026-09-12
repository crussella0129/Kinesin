# Install and use Kinesin

Install the command once, then run `kinesin` from any folder. Its terminal entry
introduces the session and asks which folder to work in. First use also saves
your model connection; later launches reuse those settings and still let you
choose the working folder. A running model server is required for answers.

Choose the [Windows installation](#windows-powershell) or [Linux installation](#linux),
then follow [your first session](#your-first-session). The
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

Choose a project folder when asked. You do not need to run Cargo or copy a
configuration into that folder. During development, `cargo run --locked` in
the checkout opens the same terminal entry.

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
memory pressure on Nighthawk. The same installed command opens the folder
selector on Debian; no Windows path or checkout-local configuration is needed.

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

On first use, provide the **model server URL** and **served model ID**. The defaults
are `http://127.0.0.1:8080` and `kinesin-qwen25-coder-7b`, matching the
[Nighthawk connection](#use-the-existing-nighthawk-model) below. Press Enter to
accept each only when that is the server you are using.

The session shows the actual folder, model and permitted actions. The personal
profile initially allows listing, reading, searching, creating folders, writing
files and editing files. It does not grant arbitrary command execution. File
paths in your requests refer to the selected folder.

For example:

```text
> make a folder called test 1
> list the files in this folder
```

You see tool activity and the model's readable answer. Folder creation must be a
real tool operation inside the chosen folder; the model's claim alone is not
verification. An existing folder is reported without replacement. Parent folders
must already exist. No command shell is needed to create a directory.

| Session command | Use |
| --- | --- |
| `/help` | Show available session commands |
| `/status` | Inspect the latest run and its acceptance status |
| `/permissions` | Show the configured tools and command grants |
| `/new` or `/clear` | Start a new thread of requests |
| `/exit` | Leave the session |

Ctrl+C cancels and stops the session. Each request is journaled as an immutable
run. Current follow-ups carry the previous answer, not a full conversation
transcript. A freeform result has no independent acceptance checker; that is
shown under `/status` and is distinct from an execution failure.

### Where settings live

| Platform | Personal settings | Private journal |
| --- | --- | --- |
| Windows | `%APPDATA%\Kinesin\settings.toml` | `%LOCALAPPDATA%\Kinesin\state\kinesin.sqlite` |
| Linux | `$XDG_CONFIG_HOME/kinesin/settings.toml`, default `~/.config/kinesin/settings.toml` | `$XDG_STATE_HOME/kinesin/kinesin.sqlite`, default `~/.local/state/kinesin/kinesin.sqlite` |

Setup creates new private directories/files and preserves existing settings and
permissions. The selected folder is applied to the current session without
rewriting the saved profile. Edit the personal TOML to change the model or tool
grants for later sessions. The configured model server is started separately.

### Explicit and machine entry

`kinesin --config PATH` uses that profile's configured workspace and skips the
folder selector. It does not silently enlarge that profile's permissions.

`kinesin --json --config PATH` emits session JSON Lines for automation.
Piped input skips setup prompts and uses an explicit configuration or the
current directory's `kinesin.toml`; it does not unexpectedly enter a setup wizard.
One-shot, batch, inspect/export/replay and service commands keep their structured
interfaces. There is no `kinesin run` subcommand.

## Start or connect a model server

Kinesin does not contain model weights or start `llama-server`. Keep the model
server running in another terminal while using the harness. The first-run defaults and explicit example use:

| Setting | Required value for the supplied example |
| --- | --- |
| URL | `http://127.0.0.1:8080` |
| Served model alias (`model_id`) | `kinesin-qwen25-coder-7b` |
| Context / slots | 4096 tokens / 1 slot |
| Tested combination | llama.cpp b6500 + Qwen2.5-Coder-7B-Instruct Q4_K_M |

`local` is the **configuration alias**, used by `--model local`. The longer
`model_id` is the server's API identity. Calling an alias `local` does not require
the GPU to be on this computer. Enter your endpoint and model ID during first-run setup, or update your personal
settings when changing models. Explicit example configurations can still be edited
separately. Changing an alias alone does not establish compatibility.

### Use the existing nighthawk model

On the Windows computer, open a separate PowerShell terminal:

```powershell
tailscale ssh charles@nighthawk
```

In that Linux shell, the files retained from the verified deployment are:

```bash
cd /home/charles/kinesin-s10-validation
./runtime/build/bin/llama-server -m ./qwen2.5-coder-7b-instruct-q4_k_m.gguf --host 127.0.0.1 --port 18080 --alias kinesin-qwen25-coder-7b -c 4096 -np 1 --jinja --no-context-shift --device Vulkan0 -ngl 99 -t 4 -tb 4 --no-webui
```

Leave it running. In a **second Windows terminal**, open the authenticated local
forward and leave that running too:

```powershell
tailscale ssh charles@nighthawk -N -T -o ExitOnForwardFailure=yes -L 127.0.0.1:8080:127.0.0.1:18080
```

In a **third Windows terminal**, test the forwarded endpoint:

```powershell
Invoke-RestMethod http://127.0.0.1:8080/health
```

Wait for `status: ok`, then run Kinesin in this third terminal. The two server/
tunnel terminals need to remain open. Stop each with Ctrl+C when finished.
Use exactly `charles@nighthawk`, with no backslash before `@`. Tailscale SSH must
be enabled on the host and allowed by your tailnet policy; sharing a tailnet alone
does not configure SSH access. A direct `http://nighthawk:18080` URL is neither
the intended listener nor accepted by Kinesin's remote-origin policy.

If Kinesin itself runs on nighthawk, no tunnel is needed. Enter
`http://127.0.0.1:18080` at the first-use **Model server URL** prompt, or set
`base_url` to that address in your [personal settings](#where-settings-live).
For an explicit profile, change its `base_url` instead. Alternatively, launch
the server with `--port 8080` to use the default URL unchanged. Check health with:

```bash
curl --fail http://127.0.0.1:18080/health
```

That checks the retained deployment's port 18080. If you instead launched with
`--port 8080`, use `curl --fail http://127.0.0.1:8080/health` and keep the default URL.

### Supply a model on another computer

The nighthawk paths above are existing deployment paths, not files supplied by
Git. For another machine, download and extract the appropriate official
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

Locate `llama-server.exe` or `llama-server` in the extracted archive. Use the same
flags as the nighthawk command, substitute your GGUF path, and use port 8080 for
the unchanged starter. On PowerShell, a quoted executable path requires `&`:

```powershell
& 'C:\path\to\llama-server.exe' -m 'C:\path\to\model.gguf' --host 127.0.0.1 --port 8080 --alias kinesin-qwen25-coder-7b -c 4096 -np 1 --jinja --no-context-shift -ngl 99 -t 4 -tb 4 --no-webui
```

The older pinned runtime is a measured compatibility baseline, not a claim that
all versions/models behave alike. See [model preflight](model-preflight.md) for
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
| Folder contains private settings/history | Choose a project subfolder outside Kinesin's settings/state directories. |
| Folder already exists | Directory creation does not replace existing entries. Pick another name or use the existing directory. |
| Model connection failure | Start the model and SSH forward if needed; check `/health`, served model identity, context and slots. |
| Explicit session has multiple models/workspaces | Use a profile with one of each or a one-shot command selecting aliases. |
| Unexpected JSON | Remove `--json` for human sessions. One-shot/operator commands intentionally remain structured. |
| State is locked | Exit the other Kinesin session using that same state. |
| Replay capture is unavailable | Select `--capture replay` when creating the original run; export cannot invent missing inputs. |
| Compiler/linker or disk-full error | Resolve build prerequisites or disk capacity, then rerun the installation command. |
