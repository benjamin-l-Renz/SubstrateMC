# WebSocket API

## Overview

The WebSocket API provides a way to establish a two-way communication channel between a client and a server over a single, long-lived connection. This API is designed to be used in web applications where real-time communication is required.

## Usage

To use the WebSocket API, you need to create a WebSocket object and connect to the server.
The current api accepts json or messagepack format which should look like the following:

```json
{
  "action": "start_server",
  "server_id": 1
}
```

The action can be one of the following:

- `StartServer`: Starts a new server with the given server id.
- `StopServer`: Stops the server with the given server id.
- `Fail`: Does nothing.

This allows us to implement the API in almost any language:

<details>
<summary>JS</summary>

```javascript
const socket = new WebSocket("ws://localhost:8080/api/ws");

socket.addEventListener("open", () => {
  console.log("Connected");

  socket.send(
    JSON.stringify({
      action: "StartServer",
      server_id: 42,
    }),
  );
});
```

</details>

<details>
<summary>Python</summary>

```python
import asyncio
import websockets
import json

async def main():
    uri = "ws://localhost:8080/api/ws"
    async with websockets.connect(uri) as ws:
        msg = {"action": "StartServer", "server_id": 42}
        await ws.send(json.dumps(msg))
        response = await ws.recv()
        print("Received:", response)

asyncio.run(main())
```

</details>

<details>
<summary>Go</summary>

```go
package main

import (
    "encoding/json"
    "log"
    "github.com/gorilla/websocket"
)

func main() {
    c, _, err := websocket.DefaultDialer.Dial("ws://localhost:8080/api/ws", nil)
    if err != nil {
        log.Fatal("dial:", err)
    }
    defer c.Close()

    msg := map[string]interface{}{"action": "StartServer", "server_id": 42}
    c.WriteJSON(msg)

    var resp map[string]interface{}
    c.ReadJSON(&resp)
    log.Println("Received:", resp)
}

```

</details>
