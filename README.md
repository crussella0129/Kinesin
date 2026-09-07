# Kinesin

The *even tinier* General Purpose Harness (but a very hard worker for its size!)

## General Goal

To create a general purpose harness as minimally, and with as few dependencies, as possible. The following global structure is proposed:

**Kineserve** (llama serve (llama.cpp) bound to local host 127.0.0.1:8080 or 8000)<-**Koil** (Custom Wireguard Connector)-> **K-Core**: Contains harness's general ReAct function routing with the below components (and for conveyance back to llama serve) + configurable firing instructions.

## Project Scaffolding
  
`*note that "User" could be a human or another agent in this context`

`Kinesin/`
`├── .github/                         # CI/CD automation workflows`
`├── src/                             # Core Rust implementation`
│   └── main.rs                      # Application entry point & orchestration
|   └── **Kineserve/**
|        ├── main.rs                 # entry point & orchestration + structs that check for installation and/or run llama.cpp, launch llama serve, and point it at model/
|        ├── input.rs                # Structs that map where the user's inputs are collected and treats that output as an immutable variable, invoked when inputs are to be collected using only std library components (io, fs, etc...) in as minimal of code as possible.
|        ├── scripts/                # Cross-platform orchestration
│            └── run-harness.ps1     # Automated Windows/WSL bootstrapper
|        └── Cargo.toml              # Zero-dependency package manifest
|   └── **Koil/**
|        ├── main.rs                #  
|        └── Cargo.toml             # Zero-dependency package manifest
|   └── **K-Core/**
|        ├── main.rs                        # structs that map the firing order of 
|        ├── input.rs                       # Structs that map where the user's inputs are collected and treats that output as an immutable variable, invoked when inputs are to be collected using only std library components (io, fs, etc...) in as minimal of code as possible.
|        └── Cargo.toml             # Zero-dependency package manifest
├── scripts/                         # Cross-platform orchestration
│   └── run-harness.ps1              # Automated Windows/WSL bootstrapper to check if the operating environment is a windows operating system, check/install WSL via winget,
├── models/
|   └── your-gguf-here.gguf
│   └── hf.rs/                       # a py03 connector that allows you 
├── Kinesin.toml                     # TOML config header (for the configurability of functionality listed above) + agent instructions in the markdown area - think 'claude.md meets deterministic config file'. This document becomes an immutable source of truth that the .toml can be compared against after compilation.
├── Cargo.toml                       # Zero-dependency package manifest - kept separate from Kinesin.toml to prevent incorrect changes and breakages (we can review that though).
└── README.md                        # Documentation`

