# Install and use Kinesin

Kinesin is a terminal application. There is no desktop window to open. You run
the `kinesin` executable; it connects to a model server that you start separately.
The model can run on this computer or another machine such as nighthawk.

Choose [Windows PowerShell](#windows-powershell) or [Linux](#linux), then complete
[model connection](#start-or-connect-a-model-server) and [first use](#your-first-run).
Commands use the development branch `dev`; an existing checkout can be used in
place of cloning again. The Rust compiler is needed to build/install from source,
not each time you run an already-built executable. Allow several GiB for builds.

## Windows PowerShell

### 1. Prerequisites and checkout

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

### 2. Prepare configuration and example data

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

### 3. Optional: make `kinesin` available by name

Cloning or `cargo run` does **not** install a command on PATH. To install only
the product executable from the checkout:

```powershell
cargo install --locked --path . --bin kinesin
$env:Path = "$HOME\.cargo\bin;$env:Path"
Get-Command kinesin
kinesin --help
```

The default install path is `C:\Users\YOUR_NAME\.cargo\bin\kinesin.exe`.
The assignment above updates this PowerShell session. Rustup normally adds that
directory to your user PATH for new terminals; if necessary add it through
**Edit environment variables for your account → Path → New**, then reopen
PowerShell. These locations assume you have not customized `CARGO_HOME` or the
install root. `cargo install kinesin` is not the procedure: this package is not
published to crates.io.

To avoid installing, use `cargo run --locked -- ...`, or after building:

```powershell
cargo build --locked --bin kinesin
.\target\debug\kinesin.exe --help
```

PowerShell requires `'.\'` to run an executable in the current directory.
For an optimized standalone build use `cargo build --locked --release --bin
kinesin`, then `.\target\release\kinesin.exe`. Installing uses a release build
by default; it can take longer than the initial debug build.
For a quicker development install that reuses the debug build, append `--debug`
to the `cargo install` command. This is the install profile used by the smoke checks.

## Linux

### 1. Prerequisites and checkout

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

### 2. Prepare configuration and example data

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

### 3. Optional: install the command

```bash
cargo install --locked --path . --bin kinesin
export PATH="$HOME/.cargo/bin:$PATH"
command -v kinesin
kinesin --help
```

The default executable is `$HOME/.cargo/bin/kinesin`. Rustup normally arranges
PATH for future shells; sourcing `$HOME/.cargo/env` fixes the current shell.
Customized `CARGO_HOME`/install roots use their own `bin` directory. No sudo is
needed for a user installation.
Append `--debug` for a quicker development install using the existing debug
build; the documented smoke checks use this profile. Omit it for the default
optimized release installation.

Without installation, use `cargo run --locked -- ...`, or:

```bash
cargo build --locked --bin kinesin
./target/debug/kinesin --help
```

Use `--release` when building an optimized executable at
`./target/release/kinesin`. Run Linux builds on a native Linux filesystem; the
sprint 10 WSL experiment recorded confined command execution failing from a
Windows-mounted build directory. This starter does not grant command execution.

## Start or connect a model server

Kinesin does not contain model weights or start `llama-server`. Keep the model
server running in another terminal while using the harness. The starter expects:

| Setting | Required value for the supplied example |
| --- | --- |
| URL | `http://127.0.0.1:8080` |
| Served model alias (`model_id`) | `kinesin-qwen25-coder-7b` |
| Context / slots | 4096 tokens / 1 slot |
| Tested combination | llama.cpp b6500 + Qwen2.5-Coder-7B-Instruct Q4_K_M |

`local` is the **configuration alias**, used by `--model local`. The longer
`model_id` is the server's API identity. Calling an alias `local` does not require
the GPU to be on this computer. Edit your copied `kinesin.toml` if using another
verified endpoint; changing its name alone does not establish compatibility.

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

If Kinesin itself runs on nighthawk, no tunnel is needed. Set the copied
configuration's `base_url` to `http://127.0.0.1:18080`, or launch the server with
`--port 8080` to use the starter unchanged. Check health with:

```bash
curl --fail http://127.0.0.1:18080/health
```

That checks the retained deployment's port 18080. If you instead launched with
`--port 8080`, use `curl --fail http://127.0.0.1:8080/health` and keep the starter URL.

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

## Your first run

Return to the checkout containing your `kinesin.toml`. These Cargo commands work
in both PowerShell and Bash:

```text
cargo run --locked -- --workspace practice --model local --prompt "Say hello" --allow-unchecked
```

Successful output is one JSON line containing `kind: "run"`, `phase: "completed"`,
`acceptance_status: "unchecked"`, `task_accepted: false`, and the answer in
`result`. The text itself varies. `--allow-unchecked` permits exit 0 for that
completed freeform result; it does not turn it into checked acceptance. Read the
exit status with `$LASTEXITCODE` in PowerShell or `echo $?` in Bash.

For an interactive session:

```text
cargo run --locked
```

Wait for the `> ` prompt, type `What does project.txt say?`, and press Enter.
Each entry creates a separate immutable run. Only the previous answer is carried
into the next entry, not the complete conversation history. End with Ctrl+C.
The session requires exactly one workspace and one model in the configuration.

If you installed the command, replace `cargo run --locked --` with `kinesin`:

```text
kinesin --workspace practice --model local --prompt "Say hello" --allow-unchecked
kinesin
```

Bare `kinesin` reads **the current directory's** `kinesin.toml`; installation does
not make it search your checkout or home directory. From another directory:

```powershell
kinesin --config "$HOME\Kinesin\kinesin.toml"
```

```bash
kinesin --config "$HOME/Kinesin/kinesin.toml"
```

All paths inside TOML resolve relative to that file, so the configured workspace
and state still work. There is **no `kinesin run` subcommand**: a single run uses
flags directly, and bare `kinesin` opens the session.

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
| PowerShell says `kinesin` is not recognized / Bash says command not found | It is not installed on this shell's PATH. Use Cargo from the checkout, the explicit built path, or the install/PATH steps above. |
| Cargo cannot determine which binary to run | Your checkout predates the default-run fix. Use `cargo run --bin kinesin -- --help`; update your checkout when ready. |
| Cannot load `kinesin.toml` | Create it using the setup block, run from its directory, or supply its absolute path with `--config`. |
| Workspace cannot be resolved | Create the configured directory; remember TOML paths are relative to the config file. |
| Model connection/readiness failure | Start the model and, if remote, the SSH forward; check `/health`, model alias, context and slot settings. Installing Kinesin does not start inference. |
| Session requires exactly one model/workspace | Use a single-run command with explicit `--model`/`--workspace`, or a configuration with one of each. |
| `run` or an unknown option is rejected | Use the exact syntax from `kinesin --help`; there is no `run` verb. |
| Replay reports unavailable capture | The original run used metadata capture or unsupported semantics; create a new run with `--capture replay`. Export cannot manufacture missing inputs. |
| State is already locked | Stop the other Kinesin controller/session using that same state directory first. |
| Build fails with disk-full or compiler/linker errors | Free space for the build, check the OS prerequisites above, and retry the same command; do not interpret a failed build as an installed program. |
