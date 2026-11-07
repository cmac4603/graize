# Graize Helm chart

This chart installs the controller API and frontend in one Helm release. Both apps
have internal `ClusterIP` Services. The controller has no Ingress; the frontend
Ingress is optional and disabled by default.

## Images and GitHub Actions

Build either image from the repository root:

```sh
docker build -f controller/Dockerfile -t ghcr.io/cmac4603/graize/controller:latest .
docker build -f frontend/Dockerfile -t ghcr.io/cmac4603/graize/frontend:latest .
```

The workflows in `.github/workflows/` publish each image to GHCR on releases,
tagged with `latest` and the full commit SHA.
Chart values default to the `latest` tags; override the tag to pin a
specific commit.

## Install or upgrade

```sh
helm upgrade --install graize .helm-charts \
  --namespace graize \
  --create-namespace
```

To pin an image to a workflow's commit-SHA tag:

```sh
helm upgrade --install graize .helm-charts \
  --namespace graize \
  --create-namespace \
  --set controller.image.tag=<full-sha> \
  --set frontend.image.tag=<full-sha>
```

For a private GHCR package, create a pull secret and pass its name in
`imagePullSecrets`:

```sh
helm upgrade --install graize .helm-charts \
  --namespace graize \
  --set 'imagePullSecrets[0].name=ghcr-creds'
```

## Worker API authentication

When the worker API uses HTTP Basic Auth, create a Kubernetes Secret outside this
chart and reference it for the controller. The Secret must be in the release
namespace. The chart does not create or manage the Secret.

```sh
helm upgrade --install graize .helm-charts \
  --namespace graize \
  --set controller.workerAuth.existingSecret=opencode-server \
  --set controller.workerAuth.passwordKey=OPENCODE_SERVER_PASSWORD
```

The controller reads the password from the referenced key and sends Basic Auth
to the worker. If the Secret also contains a username, set
`controller.workerAuth.usernameKey` to that key; otherwise, the worker's default
username (`opencode`) is used. Leave `controller.workerAuth.existingSecret`
empty to run without worker authentication.

The chart defaults to `ghcr.io/cmac4603/graize/controller:latest` and
`ghcr.io/cmac4603/graize/frontend:latest`. Set each app's `image.repository`,
`image.tag`, and `image.pullPolicy` to override them.

## Service addresses

For the example release name `graize`, workloads in the release namespace can
call the controller at `http://graize-controller:3000` and the frontend at
`http://graize-frontend`. From another namespace, use
`http://graize-controller.graize.svc.cluster.local:3000` and
`http://graize-frontend.graize.svc.cluster.local`. For a different installation,
use `<release-name>-controller` and `<release-name>-frontend`; in cross-namespace
DNS names, use the release namespace in place of `graize`. The controller
`/healthz` endpoint returns its health status and remains internal-only.

## Frontend Ingress

Ingress is disabled by default. Configure a hostname (and TLS if desired) in
`frontend.ingress`, then enable it:

```sh
helm upgrade --install graize .helm-charts \
  --namespace graize \
  --set frontend.ingress.enabled=true \
  --set 'frontend.ingress.hosts[0].host=frontend.example.test' \
  --set 'frontend.ingress.hosts[0].paths[0].path=/' \
  --set 'frontend.ingress.hosts[0].paths[0].pathType=Prefix'
```

An Ingress controller must already be installed in the cluster.

## SQLite persistence

The frontend runs as one replica because it uses SQLite. By default the chart creates
a `1Gi` `ReadWriteOnce` PVC and mounts it at `/data`; the database URL is
`sqlite:/data/data.db`. Set `frontend.persistence.storageClassName`, `size`, or
`accessModes` to suit the cluster, or set `frontend.persistence.existingClaim` to use
an existing claim. Disabling persistence uses an ephemeral `emptyDir`.

The PVC is retained on `helm uninstall` by default to protect database contents.
Delete the `<release-name>-frontend-data` PVC manually when intentionally removing
the stored data. A migration init container applies checked-in Toasty
migrations before the frontend starts.
