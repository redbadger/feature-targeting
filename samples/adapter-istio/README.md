## Install feature targeting adapter

- Create a cluster and install istio (see [readme](../init-cluster/README.md))

- Deploy echo server (see [readme](../echo-server/README.md))

- Deploy `feature-targeting` adapter:

```bash
kustomize build . | kubectl apply -f -
```
