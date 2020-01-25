#!/usr/bin/env bash

kubectl -n istio-operator get IstioControlPlane example-istiocontrolplane -o=json | jq '.metadata.finalizers = null' | kubectl delete -f -
kubectl delete ns istio-operator --grace-period=0 --force
kubectl delete ns istio-system --grace-period=0 --force
