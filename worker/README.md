# OpenCode worker image

This image runs the pinned OpenCode v2.1.2 headless HTTP server. It listens on
port `4096`, binds to `0.0.0.0`, and uses `/workspace` as its working directory.
The process runs as UID/GID `10001`. Mount the task's checkout at `/workspace`
and make it writable by that UID (for Kubernetes volumes, configure suitable
`fsGroup` permissions).

## Build and run

Build from the repository root because this is the worker image's build context:

```sh
docker build -f worker/Dockerfile -t ghcr.io/cmac4603/graize/worker:latest .
```

Run it with a writable workspace and an OpenCode server password:

```sh
docker run --rm \
  -p 127.0.0.1:4096:4096 \
  -e OPENCODE_SERVER_PASSWORD="$OPENCODE_SERVER_PASSWORD" \
  -v "$PWD:/workspace" \
  ghcr.io/cmac4603/graize/worker:latest
```

On each GitHub release, the publish workflow pushes `latest` and a
commit-SHA-tagged image to `ghcr.io/cmac4603/graize/worker`.

In Kubernetes, inject `OPENCODE_SERVER_PASSWORD` from a Secret. OpenCode v2.1.2
protects the server API with HTTP Basic Auth when this variable is set; the
username defaults to `opencode`. `OPENCODE_SERVER_USERNAME` can override it. Do
not expose port `4096` publicly. The controller should connect over the Pod
network (for example, to the worker Pod IP and port `4096`).

Provider credentials are separate from the server password. Supply the chosen
model provider's API key using its supported environment variable or mount an
OpenCode `auth.json` file. Do not bake credentials into this image.

## Communicate with the worker (OpenCode v2 API)

The worker runs OpenCode `2.1.2`. Its HTTP API uses `/api/...` routes. The
interactive UI endpoints are not needed for a controller client. Set the worker
base URL and password for the examples:

```sh
export WORKER_URL=http://127.0.0.1:4096
export OPENCODE_SERVER_PASSWORD='the-value-in-the-worker-secret'
```

Check readiness:

```sh
curl --fail --user "opencode:$OPENCODE_SERVER_PASSWORD" \
  "$WORKER_URL/api/info"
```

Create a session in `/workspace`. The response includes the session ID at
`.data.id`:

```sh
SESSION_ID="$(curl --fail --silent --show-error \
  --user "opencode:$OPENCODE_SERVER_PASSWORD" \
  --header 'Content-Type: application/json' \
  --data '{"title":"task-123","location":{"directory":"/workspace"}}' \
  "$WORKER_URL/api/session" | jq -r '.data.id')"
```

Submit work to that session:

```sh
curl --fail --silent --show-error \
  --user "opencode:$OPENCODE_SERVER_PASSWORD" \
  --header 'Content-Type: application/json' \
  --data '{"text":"Inspect the repository and implement the requested change."}' \
  "$WORKER_URL/api/session/$SESSION_ID/prompt"
```

The prompt endpoint durably accepts the task and schedules agent execution; its
response is the accepted user input, not the final assistant answer. Wait for
the session to become idle, then fetch its active context:

```sh
curl --fail --silent --show-error --request POST \
  --user "opencode:$OPENCODE_SERVER_PASSWORD" \
  "$WORKER_URL/api/experimental/session/$SESSION_ID/wait"

curl --fail --silent --show-error \
  --user "opencode:$OPENCODE_SERVER_PASSWORD" \
  "$WORKER_URL/api/session/$SESSION_ID/context"
```

The wait endpoint is currently marked experimental. For live progress, OpenCode
also provides the server-sent-events endpoint `GET /api/event`; use session
events to follow execution. The running server's `/openapi.json` endpoint
describes the exact API exposed by the pinned image.

## GitHub MCP server

The worker does not need Docker-in-Docker or a separately installed local MCP
container to use GitHub's hosted MCP server. Add this to the workspace's
`opencode.json` and inject `GITHUB_PERSONAL_ACCESS_TOKEN` from a Kubernetes
Secret:

```json
{
  "$schema": "https://opencode.ai/config.json",
  "mcp": {
    "github": {
      "type": "remote",
      "url": "https://api.githubcopilot.com/mcp/",
      "enabled": true,
      "oauth": false,
      "headers": {
        "Authorization": "Bearer {env:GITHUB_PERSONAL_ACCESS_TOKEN}"
      }
    }
  }
}
```

OpenCode loads this configuration when the worker starts. The PAT is supplied
at runtime and is not stored in the image. Limit the PAT's permissions to the
repositories and operations the agent needs.

## References

- [OpenCode v2 Docker image](https://opencode.ai/v2/docs#docker)
- [OpenCode v2 server docs](https://opencode.ai/v2/docs/server)
- [OpenCode v2 SDK docs](https://opencode.ai/v2/docs/sdk)
- [OpenCode session API definitions](https://github.com/anomalyco/opencode/blob/dev/packages/protocol/src/groups/session.ts)
- [OpenCode prompt input schema](https://github.com/anomalyco/opencode/blob/dev/packages/schema/src/prompt-input.ts)
- [GitHub MCP Server setup for OpenCode](https://github.com/github/github-mcp-server/blob/main/docs/installation-guides/install-opencode.md)
