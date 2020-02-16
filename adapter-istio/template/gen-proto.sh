#!/usr/bin/env bash

set -euxo pipefail

##### ADAPTER

# compile config proto
protoc -o config.proto_descriptor \
  --include_imports \
  config.proto

# generate adapter manifest for Istio
mixgen adapter \
  --config config.proto_descriptor \
  --session_based=false \
  --name feature-targeting \
  --description "Mixer adapter for feature targeting" \
  --templates feature-targeting \
  --output ../../samples/adapter-istio

##### TEMPLATE

# compile template proto
protoc -o template.proto_descriptor \
  -I ./ \
  -I ./proto \
  -I ./proto/common-protos \
  --include_imports \
  template.proto

# add mixer api: creates generated_template.proto
mixgen api -t template.proto_descriptor --go_out /dev/null

# compile modified template
protoc -o generated_template.proto_descriptor \
  -I ./ \
  -I ./proto \
  -I ./proto/common-protos \
  --include_imports \
  generated_template.proto

# generate template manifest for Istio
mixgen template \
  --descriptor generated_template.proto_descriptor \
  --name feature-targeting \
  --output ../../samples/adapter-istio/feature-targeting-template.yaml
