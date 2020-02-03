# Before building

## Install Mixgen and Istios protocol definitions

```sh
go get -u -v istio.io/istio/mixer/tools/mixgen
go install istio.io/istio/mixer/tools/mixgen
```

Download istio protobuf definitions

```sh
cd template
mkdir -p proto
cd proto

curl -LO https://github.com/istio/api/archive/1.4.3.tar.gz
gunzip 1.4.3.tar.gz
tar xvf 1.4.3.tar.gz

cd ..
```

## Compile the template protobuf

```sh
protoc -o template.proto_descriptor \
  --cpp_out=. \
  -I ./ \
  -I ./proto \
  -I ./proto/common-protos \
  template.proto
```

## Generate Mixer adapter compatible resources

```sh
mixgen api -t template.proto_descriptor --go_out /dev/null
```

TODO: Custom K8s resources
