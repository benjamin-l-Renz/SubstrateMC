# Delete Server

Deletes an existing minecraft server via HTTP POST request (/api/remove_server)

```http
POST /api/remove_server
```

## Request Format

Send a small json payload 

```json
{
  "name": "test"
}
```

## Fields

### **name**

**Type:** `string` **Required**

the server name needed to delete a server
