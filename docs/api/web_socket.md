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

- `start_server`: Starts a new server with the given server id.
- `stop_server`: Stops the server with the given server id.
- `get_console_output`: Get the console output of a running server using an id.
- `send_command`: Send a command to stdin of a running server using an id and a command.
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
      action: "start_server",
      server_id: 42,
    }),
  );
});
```
