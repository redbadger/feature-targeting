# Before building

## Install Mixgen and Istio's protocol definitions

Install `mixgen`

```sh
go get -u -v istio.io/istio/mixer/tools/mixgen
go install istio.io/istio/mixer/tools/mixgen
```

Download istio protobuf definitions

```sh
(cd template \
  && ./get-proto.sh)
```

## Compile the proto descriptors and istio manifests

```sh
(cd template \
  && ./gen-proto.sh)
```

Validate the generated manifests:

```sh
kustomize build ../samples/adapter-istio | istioctl validate -f -
```
