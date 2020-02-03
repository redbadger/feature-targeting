# Initialize a local Kubernetes cluster to run the samples

This uses [kind](https://kind.sigs.k8s.io/) to provision a cluster. You also need to install
[Istio](https://istio.io/), as the only available implementation of feature targeting is
currently an Istio (specifically Mixer) adapter.

## Create a cluster

Install kind.

```sh
$ brew install kind
(...)
==> Summary
🍺  /usr/local/Cellar/kind/0.7.0: 7 files, 8.9MB
```

Start a cluster with the provided script

```sh
$ ./kind.sh
Creating cluster "kind" ...
 ✓ Ensuring node image (kindest/node:v1.17.0) 🖼
 ✓ Preparing nodes 📦
 ✓ Writing configuration 📜
 ✓ Starting control-plane 🕹️
 ✓ Installing CNI 🔌
 ✓ Installing StorageClass 💾
Set kubectl context to "kind-kind"
You can now use your cluster with:

kubectl cluster-info --context kind-kind

Have a nice day! 👋
```

## Install Istio

Follow the installation guide at <https://istio.io/docs/setup/getting-started/>

Then run the provided script:

```sh
$ ./istio.sh
Checking the cluster to make sure it is ready for Istio installation...
(...)
Install Pre-Check passed! The cluster is ready for Istio installation.
(...)
✔ Installation complete
```

### Remove Istio

If needed, you can remove istio with

```sh
$ istio-delete.sh
```
