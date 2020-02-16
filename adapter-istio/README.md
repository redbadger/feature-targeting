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

## Compile the template protobuf

```sh
(cd template \
  && protoc -o template.proto_descriptor \
    --cpp_out=. \
    -I ./ \
    -I ./proto \
    -I ./proto/common-protos \
    --include_imports \
    template.proto \
  && rm template.pb.cc template.pb.h)
```

## Generate Mixer adapter compatible resources

```sh
(cd template \
  && mixgen api -t template.proto_descriptor --go_out /dev/null)
```

## Kubernetes resources

```sh
(cd template \
  && mixgen template \
    --descriptor template.proto_descriptor \
    --name feature-targeting \
    --output ../../samples/adapter-istio/feature-targeting-template.yaml \
  && mixgen adapter \
    --config template.proto_descriptor \
    --session_based=false \
    --name feature-targeting \
    --description "Mixer adapter for feature targeting" \
    --templates feature-targeting \
    --output ../../samples/adapter-istio)
```
