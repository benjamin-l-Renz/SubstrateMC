# ElectronMC

> **ElectronMC is under active development and not yet ready for production use.**

**ElectronMC** is a high-performance, lightweight Minecraft server interface written entirely in **Rust**.
It exposes a modern and clean **HTTP and WebSocket API** that lets you manage Minecraft servers efficiently — **in real time**.

ElectronMC acts as a **bridge between your application and Minecraft servers**, making it easy to build:

- Web dashboards and control panels
- Automation and monitoring tools
- CLI applications
- Custom Minecraft hosting platforms

Built with developers in mind, ElectronMC offers a **robust and flexible architecture** that integrates easily with almost any programming language or framework using simple HTTP requests.

---

## Table of Contents

- [Features](#features)
- [Why ElectronMC](#why-electronmc)
- [Who is ElectronMC for](#who-is-electronmc-for)
- [Why not use scripts or manual control](#why-not-use-scripts-or-manual-control)
- [Quick Start](#quick-start)
- [Documentation](#documentation)
- [Contributing](#contributing)

---

## Features

- [x] Create Minecraft servers
- [x] Start Minecraft servers
- [x] Stop Minecraft servers
- [ ] Stream console output
- [ ] View logs
- [ ] Mod loader support
- [ ] Delete Servers
- [ ] Server Statistics
- [ ] Editing Config

---

## Why ElectronMC?

- **Blazingly fast** — powered by Rust
- **Real-time control** — WebSocket-based event streaming
- **Clean API design** — predictable and easy to work with
- **Language-agnostic** — usable from virtually any language

---

## Who is ElectronMC for?

ElectronMC is designed for **developers, system administrators, and creators** who want modern, programmatic control over Minecraft servers.

It is intentionally **approachable**, even if you only have **basic programming experience**.
If you can send an HTTP request, you can use ElectronMC.

---

## Why not use scripts or manual control?

| Scripts / Manual Control | ElectronMC                 |
| ------------------------ | -------------------------- |
| Hard to scale            | Built for growth           |
| No real-time feedback    | Live WebSocket events      |
| Fragile process handling | Safe, structured control   |
| Hard to integrate        | Clean HTTP API             |
| Error-prone              | Strong typing & validation |

ElectronMC replaces scripts with a **single, reliable control layer** for Minecraft servers.

---

## Quick Start

Download ElectronMC from the **GitHub releases page** and extract the archive:

```bash
tar -xvf electron-mc.tar.gz # Linux

cd electron-mc

./electron-mc
```

This will start the ElectronMC server locally on **port 8080**.

Create a new Minecraft server by sending a `POST` request to the `/servers` endpoint:

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

---

## Documentation

ElectronMC provides a comprehensive **documentation** covering all aspects of its usage.
You can find it on the **GitHub repository** under the **docs** directory.

---

## Contributing

ElectronMC welcomes contributions from the community.
If you're interested in contributing, please read our [contributing guidelines](CONTRIBUTING.md).
