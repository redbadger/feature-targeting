# Step 2: Install feature targeting adapter

This enables feature state injection with Istio, by attaching the feature
targeting adapter to ingress gateways.

This is the main point of configuring _how_ you would like feature targeting
to happen, mainly:

- Which parts of the request should be available to target _implicit_ features
  (i.e. the template instance configuration)
- Which methods of explicit feature targeting you'd like to enable
  (i.e. the handler configuration)
  **NOTE this is hardcoded for now**

## Before you start

- Create a cluster and install istio (see [readme](../init-cluster/README.md))
- Deploy echo server (see [readme](../echo-server/README.md))

## Deploy the adapter

```bash
kustomize build . | kubectl apply -f -
```

What have we just done?

1. Deployed the server responsible for processing requests into feature state (`feature-targeting-server.yml`)
2. Declared a protocol used by Istio Mixer to talk to our server (`feature-targeting-template.yml`)
3. Register the server as a Mixer adapter and declared its configuration shape (`feature-targeting.yml`)
4. Created a handler (adapter instance) with specific configuration (`handler.yml`)
5. Created an instance (of the template) and configured how it's populated by
   request attributes provided by Istio (`instance.yml`)
6. Attach the handler and instance to the ingress gateway proxies in Istio and
   configured how the requests get modified (`rule.yml`)

It may be helpful to read about [the architecture](https://istio.io/docs/reference/config/policy-and-telemetry/mixer-overview/) of Mixer adapters.
