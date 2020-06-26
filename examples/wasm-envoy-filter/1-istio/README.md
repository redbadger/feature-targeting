# Step 1: Get a cluster with Istio 1.6 installed

The easiest way to test is probably to use the Kubernetes cluster in Docker For Desktop (open preferences and enable Kubernetes on the Kubernetes tab). You could also use [KinD](https://kind.sigs.k8s.io/), or MicroK8s etc.

You also need to install [Istio](https://istio.io/).

## Install Istio

Follow the installation guide at <https://istio.io/docs/setup/getting-started/>.

Or, if you have the right version of `istioctl` installed, you can just run `make` in this directory.

### Remove Istio

If needed, you can remove Istio with

```sh
make delete
```
