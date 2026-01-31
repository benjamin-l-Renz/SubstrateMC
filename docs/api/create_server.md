# Create Server

Creates a new Minecraft server using a JSON configuration sent via HTTP POST request (/api/create_server).

```http
POST /api/create_server
```

## Request Format

Send a JSON payload with the following structure:

```json
{
  "name": "test",
  "minecraft_version": "1.21.1",
  "loader": "vanilla",
  "forced_java_version": "21",
  "agree_eula": true
}
```

## Fields

### **name**

**Type:** `string` **Required**

The server identifier. Used to name the server directory on the filesystem and as a unique reference.

**Example:** `"test"`, `"survival-world"`, `"pvp-arena"`

### **minecraft_version**

**Type:** `string` **Required**

The target Minecraft version for the server.

**Example:** `"1.21.1"`, `"1.20.4"`

### **loader**

**Type:** `string` **Required**

The Minecraft server implementation/loader type

**Supported values:** `"vanilla"`, `"fabric"` (depends on available loaders)

### **forced_java_version**

**Type:** `string` **Optional**

Specify a particular Java version for this server. Only set this if the automatic selection is insufficient.

**Supported values:** `"21"`, `"17"`, `"11"` (depends on available runtimes)

**Default:** Auto-selected based on Minecraft version compatibility

### **agree_eula**

**Type:** `boolean` **Optional**

Agree to the Minecraft EULA. Only set this if you agree to the EULA.

## Response

Returns an index of the server as msgpack for later use.

## Notes

- The `name` field must be unique; creating a server with an existing name will fail
- Java version will be automatically selected based on Minecraft version if not specified
- Server data is stored in the `servers/` directory
- There is no official support for java versions that are not adoptium jdk's
- Other minecraft mod or plugin loaders can easily be added over `minecraft.json
