#!/usr/bin/env bash

set -e

rm -rf proto
mkdir proto
cd proto

VERSION=1.4.4
curl -LO https://github.com/istio/api/archive/${VERSION}.tar.gz
gunzip ${VERSION}.tar.gz
tar xvf ${VERSION}.tar
mv api-${VERSION}/* .
rm -rf api-${VERSION} ${VERSION}.tar
