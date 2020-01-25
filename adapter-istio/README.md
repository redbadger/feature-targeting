# install mixgen

```bash
go install istio.io/istio/mixer/tools/mixgen

cd template
mkdir -p proto
cd proto

curl -LO https://github.com/istio/api/archive/1.4.3.tar.gz
gunzip 1.4.3.tar.gz
tar xvf 1.4.3.tar.gz

cd ..
protoc -o template.proto_descriptor \
  --cpp_out=. \
  -I ./ \
  -I ./proto \
  -I ./proto/common-protos \
  template.proto
mixgen api -t template.proto_descriptor --go_out /dev/null

```
