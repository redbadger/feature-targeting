#!/usr/bin/env bash

kubectl delete ns istio-system --grace-period=0 --force
