# echo service

This sample uses a HTTP echo service to reflect back the request made by the
client. You can test different ways of injecting and overriding features.

## What we'll show

With this example, we use an echo service to show how feature state gets
injected into the incoming requests. The features that are enabled will be
listed in the `X-Features` header of the request.

Additionally, features can be enabled explicitly, by passing the `x-feature-override`
header to the incoming request. The lists will be merged.

> TODO enable Host header based targeting as well

## Try it out

### Before you start

Install Istio and the relevant adapter (see this [readme](../README.md))

### Deploy the echo service

1. Install the echo service

```bash
make
```

1. Test the echo service

```sh
curl --resolve echo.localhost:80:127.0.0.1 -vvv http://echo.localhost
```

### Testing it out

Implicit features will be injected by the system to any request
