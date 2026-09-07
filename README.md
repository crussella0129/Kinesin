# Kinesin

The *even tinier* General Purpose Harness (but a very hard worker for its size!)

##General Goal: to create a general purpose harness as minimally, and with as few dependencies, as possible. 

##Project Scaffolding (which may also be the structure of the main function):
  *note that "User" could be a human or another agent in this context

**llama serve** (llama.cpp) **bound to local host** (127.0.0.1:8080 or 8000)<-**Custom Wireguard Connector**-> **Kinesin Core**: Contains harness's general ReAct function routing with the below components (and for conveyance back to llama serve) + configurable firing instructions.


###Kinesin Core Scaffolding
|_ ####Kinesin.toml : TOML config header (for the configurability of functionality listed above) + agent instructions in the markdown area - think 'claude.md meets deterministic config file'. The elements of this document become immutable an immutable source of truth that the .toml can be compared against after compilation as well
|_ ####ReAct.rs : structs that map the firing order of 
|_ ####input.rs : structs that map where the user's inputs are collected and treats that output as an immutable variable, invoked when inputs are to be collected. Process most similar to "collects keystroke input / allows typing -> 'enter' press collects that state -> sends that state immutably to files that call for the output of this file" using only std library components in as minimal of code as possible
|_ ####
|_
