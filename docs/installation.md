## Installation

## Downloading the project

> Currently not available

## Compiling the project

### 1. Install Rust

> `At the moment only Linux and macOS are supported.`

If you don’t have Rust installed, you can install it using the official Rust installer

### 3. Clone and build the project

```bash
git clone https://github.com/your-username/project.git
cd project
cargo build --release

```

### 4. Create config

There a two required configuration files: `minecraft.json` and `java.json`.
These files define the urls and versions of the Minecraft server and java runtime without them you will not be able to install
java or a minecraft server.

#### Java.json

```json
{
  "linux": [
    {
      "url": "placeholder.tar.gz", // Placeholder url
      "version": "21"
    }
  ]
}
```

#### Minecraft.json

```json
{
  "versions": {
    "1.21.11": {
      "vanilla": {
        "url": "https://example.com/server.jar", // Placeholder url
        "java_version": "21"
      }
    }
  }
}
```

### 5. Run the project

You can find the binary in the `target/release` directory.

```bash
cd target/release

# linux
./project
```
