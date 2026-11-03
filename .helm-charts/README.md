# Graize Helm chart

This chart installs the controller API as a Kubernetes Deployment and a
cluster-internal `ClusterIP` Service. It does not create an Ingress or expose
the controller outside the cluster.

## Build and publish the controller image

Run this command from the repository root:

```sh
docker build -f controller/Dockerfile -t graize/controller:0.1.0 .
```

Before installing, make the image available to the Kubernetes nodes by pushing
it to a registry they can access or by loading it into your local cluster. For
example, to publish it to a registry:

```sh
docker tag graize/controller:0.1.0 registry.example.test/graize/controller:0.1.0
docker push registry.example.test/graize/controller:0.1.0
```

## Install

Install the chart into a namespace, creating it if necessary:

```sh
helm install graize .helm-charts \
  --namespace graize \
  --create-namespace \
  --set controller.image.repository=registry.example.test/graize/controller
```

The default image is `graize/controller:0.1.0`. Set `controller.image.repository`,
`controller.image.tag`, and `controller.image.pullPolicy` to match the image
available to your cluster. The replica count, Service port, and container
resources can also be set through chart values.

## Internal controller address

Other workloads in the same namespace can call the controller at
`http://graize-controller:3000`. Workloads in another namespace can use the
fully qualified address
`http://graize-controller.graize.svc.cluster.local:3000`. The `/healthz`
endpoint returns the controller health status.

The Service name is `<release-name>-controller`, so replace `graize` in the
examples if you use another Helm release name. If you change
`controller.service.port`, use that port in the client URL as well.
