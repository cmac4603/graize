# Graize Helm chart

This chart installs the controller API and UI in one Helm release. Both apps
have internal `ClusterIP` Services. The controller has no Ingress; the UI
Ingress is optional and disabled by default.

## Images and GitHub Actions

Build either image from the repository root:

```sh
docker build -f controller/Dockerfile -t ghcr.io/graize/graize-controller:latest .
docker build -f ui/Dockerfile -t ghcr.io/graize/graize-ui:latest .
```

The workflows in `.github/workflows/` publish each image to GHCR on relevant
pushes to `main`, tagged with `latest` and the full commit SHA. They do not
deploy the chart. Chart values default to the `latest` tags; override the tag
to pin a specific commit.

GHCR packages are private unless their visibility is changed. Make both
packages public for credential-free pulls, or create a Kubernetes Docker
registry secret in the release namespace and configure `imagePullSecrets`.

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
  --set ui.image.tag=<full-sha>
```

For a private GHCR package, create a pull secret and pass its name in
`imagePullSecrets`:

```sh
helm upgrade --install graize .helm-charts \
  --namespace graize \
  --set 'imagePullSecrets[0].name=ghcr-creds'
```

The chart defaults to `ghcr.io/graize/graize-controller:latest` and
`ghcr.io/graize/graize-ui:latest`. Set each app's `image.repository`, `image.tag`,
and `image.pullPolicy` to override them.

## Service addresses

Within the release namespace, workloads can call the controller at
`http://graize-controller:3000` and the UI at `http://graize-ui`. From other
namespaces, use `http://graize-controller.graize.svc.cluster.local:3000` and
`http://graize-ui.graize.svc.cluster.local`. The controller `/healthz` endpoint
returns its health status.

Service names include the Helm release name; replace `graize` if you install
under a different release name. The controller remains internal-only.

## UI Ingress

Ingress is disabled by default. Configure a hostname (and TLS if desired) in
`ui.ingress`, then enable it:

```sh
helm upgrade --install graize .helm-charts \
  --namespace graize \
  --set ui.ingress.enabled=true \
  --set 'ui.ingress.hosts[0].host=ui.example.test' \
  --set 'ui.ingress.hosts[0].paths[0].path=/' \
  --set 'ui.ingress.hosts[0].paths[0].pathType=Prefix'
```

An Ingress controller must already be installed in the cluster.

## SQLite persistence

The UI runs as one replica because it uses SQLite. By default the chart creates
a `1Gi` `ReadWriteOnce` PVC and mounts it at `/data`; the database URL is
`sqlite:/data/data.db`. Set `ui.persistence.storageClassName`, `size`, or
`accessModes` to suit the cluster, or set `ui.persistence.existingClaim` to use
an existing claim. Disabling persistence uses an ephemeral `emptyDir`.

The PVC is retained on `helm uninstall` by default to protect database contents.
Delete the `<release-name>-ui-data` PVC manually when intentionally removing
the stored data. A migration init container applies checked-in Toasty
migrations before the UI starts.
