# Before building

## Install Mixgen and Istio's protocol definitions

Install `mixgen`

```sh
go get -u -v istio.io/istio/mixer/tools/mixgen
go install istio.io/istio/mixer/tools/mixgen
```

## Compile the proto descriptors and istio manifests

```sh
make protobuf-install
```

Validate the generated manifests:

```sh
kustomize build ../examples/adapter-istio | istioctl validate -f -
```
