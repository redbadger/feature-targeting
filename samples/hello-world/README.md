# Hello World

This sample uses a HTTP echo server to reflect back the request made by the
client. You can test different ways of injecting and overriding features.

## What we'll show

With this example, we use an echo server to show how feature state gets
injected into the incoming requests. The features that are enabled will be
listed in the `X-Features` header of the request.

Additionally, features can be enabled explicitly, by passing the `X-Features`
header to the incoming request. The lists will be merged.

> TODO enable Host header based targeting as well

## Try it out

### Before you start

1. Initialise a cluster (see `../1-init-cluster`)
2. Enable feature targeting (see `../2-enable-adapter`)

### Deploy the echo service

1. Install the echo service

```bash
kustomize build . | kubectl apply -f -
```

1. Add entry to hosts file

```sh
echo echo.localhost 127.0.0.1 | sudo tee -a /etc/hosts
```

1. Test the echo server

```sh
curl -vvv http://echo.localhost
```

### Testing it out

Implicit features will be injected by the system to any request
