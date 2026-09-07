# Kinesin

The *even tinier* General Purpose Harness (but a very hard worker for its size!)

## General Goal

To create a general purpose harness as minimally, and with as few dependencies, as possible. The following global structure is proposed:

```
Kineserve (llama serve (llama.cpp) bound to local host 127.0.0.1:8080 or 8000) <-> Koil (Custom Wireguard Connector) <-> K-Core: (Contains harness's general ReAct function routing + configurable instructions).
```

## Project Scaffolding
  
*note that "User" could be a human or another agent in this context

```
Kinesin/
├── .github/                         # CI/CD automation workflows
├── src/                             # Core Rust implementation
│   └── main.rs                      # Application entry point & orchestration
|   └── Kineserve/
|        ├── main.rs                 # entry point & orchestration + structs that launches llama serve and point it at model/
|        ├── input.rs                # Structs that map where the user's inputs are collected and treats that output as an immutable variable, invoked when inputs are to be collected using only std library components (io, fs, etc...) in as minimal of code as possible.
|        ├── scripts/                # Cross-platform orchestration
│            └── run-harness.ps1     # Automated Windows/WSL bootstrapper
|        └── Cargo.toml              # Zero-dependency package manifest
|   └── Koil/
|        ├── main.rs                 #  
|        └── Cargo.toml              # Zero-dependency package manifest
|   └── K-Core/
|        ├── main.rs                 # Entry point & orchestration + structs that map the firing order of K-Core components to Llama serve compatible JSON traces  
|        ├── input.rs                # Structs that map where the user's inputs are collected and treats that output as an immutable variable, invoked when inputs are to be collected using only std library components (io, fs, etc...) in as minimal of code as possible.
|        └── Cargo.toml              # Zero-dependency package manifest
├── scripts/                         # Cross-platform orchestration
│   └── first_flight.ps1             # Automated bootstrapper that observes environment (Windows or Linux), engages Windows/WSL bootstrapper to check if the operating environment is a windows operating system, check/install WSL, llama.cpp, wireguard (and other core, non-rust deps) via the appropriate package manager.
├── models/                          # Directory for storing models
|   └── your-gguf-here.gguf          # Your GGUF format model text-to-text of choice (coding or agent models recommeneded), selected beforehand and copied or cloned to this directory
├── traces/                          # Directory for storing JSON traces of each session
|   └── trace_hash.json              # Hash lookup table for json Traces in the logs/ directory.
|   └── logs                         # Compilation of all Traces (immutable recordings of all activity that flows across Koil (which is all activity between Kineserve and K-Core))
|         └── trace#.json            # trace id is randomly generated (key/hash), rather than sequential.
├── kinesin.toml                     # TOML config header (for the configurability of functionality listed above) + agent instructions in the markdown area - think 'claude.md meets deterministic config file'. This document becomes an immutable source of truth that the .toml can be compared against after compilation.
├── Cargo.toml                       # Zero-dependency package manifest - kept separate from Kinesin.toml to prevent incorrect changes and breakages (we can review that though).
└── README.md                        # Documentation
```

