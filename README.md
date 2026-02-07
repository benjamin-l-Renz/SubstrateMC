# SubstrateMC

> **SubstrateMC is under active development and not yet ready for production use.**

**SubstrateMC** is a high performance, lightweight Minecraft server interface written in **Rust**. It tries to expose a modern and clean **HTTP and WebSocket API** that enables you to manage Minecraft servers efficiently - in real time.

**SubstrateMC** acts as a simple bridge between your panel and the Minecraft servers this makes it easy to build minecraft panels and automation tools.

## Table of Contents

- [Features](#features)
- [Why SubstrateMC](#why-substratemc)
- [Who is SubstrateMC for](#who-is-substratemc-for)
- [Why not manual manage servers ?](#why-not-manual-manage-servers)
- [Quick Start](#quick-start)
- [Contributing](#contributing)

## Features

- [ ] Create Minecraft Servers
- [ ] Stop Minecraft Servers
- [ ] Start Minecraft Servers
- [ ] Stream Console Output
- [ ] Mod Loader Support
- [ ] Delete Servers
- [ ] Editing Configs

## Why SubstrateMC

- **Blazingly fast** — powered by Rust
- **Real-time control** — WebSocket-based event streaming
- **Clean API design** — predictable and easy to work with
- **Language-agnostic** — usable from virtually any language

## Who is SubstrateMC for

SubstrateMC is designed for developers who want to build efficient and scalable Minecraft server management systems. Whether you're creating a web dashboard, automation tool, or custom hosting platform, SubstrateMC provides a robust and flexible architecture that integrates seamlessly with your existing infrastructure.

## Why not manual manage servers ?

|           Manual Control | SubstrateMC                |
| ------------------------ | -------------------------- |
| Hard to scale            | Built for growth           |
| No real-time feedback    | Live WebSocket events      |
| Fragile process handling | Safe, structured control   |
| Hard to integrate        | Clean HTTP API             |
| Error-prone              | Strong typing & validation |

SubstrateMC is designed for running multiple Minecraft servers and easily controlling them.

## Quick Start 

Install SubstrateMC over the github releases or build it yourself.

Run the binary and make sure every config is supplied

````bash

# Run the binary
./substrate-mc

```` 
This will start the SubstrateMC server locally on **port 8080**.

Create a new Minecraft server by sending a `POST` request to the `/api/create_server` endpoint:

```bash
curl -X POST http://localhost:8080/api/create_server \
  -H "Content-Type: application/json" \
  -d '{
    "name": "test",
    "minecraft_version": "1.21.11",
    "loader": "vanilla",
    "forced_java_version": "21",
    "agree_eula": true
  }'
```

## Contributing

ElectronMC welcomes contributions from the community.
If you're interested in contributing, please read our [contributing guidelines](CONTRIBUTING.md).
