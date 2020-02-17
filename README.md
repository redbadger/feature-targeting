## Feature targeting

### Adapter for Istio

This is an experimental mixer adapter for Istio. Incoming requests are intercepted and the `method`, `path` and `features` header is forwarded to the adapter so that the `features` header can be manipulated before the request is received by a pod.

Currently, the header is split on whitespace and a new feature called `new_feature` is added to the sorted and de-duped list of features. We will evolve the adapter to add real features soon.

#### Getting started

See the [samples](./samples/README.md) directory to get started.
