# Hello World

This sample uses a HTTP echo server to reflect back the request made by the
client. As a demonstration, it uses istio's sample keyval Mixer adapter to
modify incoming headers

> TODO: switch this to the custom feature targeting adapter

## Install keyval

Deploy the service

```sh
kubectl run keyval --image=gcr.io/istio-testing/keyval:release-1.1 --namespace istio-system --port 9070 --expose
```

From the istio source folder

```sh
kubectl apply -f samples/httpbin/policy/keyval-template.yaml
kubectl apply -f samples/httpbin/policy/keyval.yaml
```

## Deploy the echo service

```sh
kustomize build ../echo-server | kubectl apply -f -
```

## Create the adapter instance

```sh
kubectl apply -f handler.yaml
kubectl apply -f instance.yaml
kubectl apply -f rule.yaml
```
