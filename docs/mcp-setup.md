# EasyTube MCP Server Setup

EasyTube can run as an [MCP (Model Context Protocol)](https://modelcontextprotocol.io/) server,
allowing AI assistants to control video downloads via standardized tool calls.

## Supported AI Clients

| Client | Config File |
|---|---|
| **opencode** | `~/.config/opencode/mcp.json` |
| **Claude Desktop** | `~/Library/Application Support/Claude/claude_desktop_config.json` |
| **Cursor** | `.cursor/mcp.json` (project) or `~/.cursor/mcp.json` (global) |

## Configuration

Add the following to your MCP server configuration:

```json
{
  "mcpServers": {
    "easytube": {
      "command": "easytube",
      "args": ["mcp"]
    }
  }
}
```

For development builds (macOS):

```json
{
  "mcpServers": {
    "easytube": {
      "command": "/Users/<your-user>/code-project/EasyTube/src-tauri/target/debug/easytube",
      "args": ["mcp"]
    }
  }
}
```

## Available Tools

| Tool | Description |
|---|---|
| `easytube_probe` | Probe a video URL. Returns title, duration, thumbnail, available formats. |
| `easytube_download` | Start a download. Returns a `job_id` for tracking. |
| `easytube_progress` | Check download progress by job_id. |
| `easytube_stop` | Cancel a running download. |
| `easytube_settings` | Read or update settings. |

## Example Conversation

```
User: Can you download this video? https://youtube.com/watch?v=dQw4w9WgXcQ

AI: Let me check what's available at that URL.
    [calls easytube_probe]

    This is "Rick Astley - Never Gonna Give You Up" (3:32).
    Available in 37 formats including 4K, 1080p, 720p, and audio.
    Should I download it in best quality?

User: Yes, best quality please.

AI: [calls easytube_download]
    Download started! job_id: abc-123-def
    [calls easytube_progress]
    Download complete: 1080p mp4 saved to ~/Downloads
```

## Testing

Verify the MCP server works:

```bash
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}' | easytube mcp
```

Expected output includes 5 tools: `easytube_probe`, `easytube_download`, etc.
