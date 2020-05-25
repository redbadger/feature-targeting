# A kubernetes operator to manage Envoy filter configuration for Feature Targeting

The Istio `EnvoyFilter` resource carries significant cognitive overhead when all you want to do is configure the [WASM filter](../adapter-proxy-wasm/README.md) for feature targeting.

This is a Kubernetes operator (written in [Rust](https://www.rust-lang.org)) that manages the configuration of specified envoy side-cars by wrapping the specified configuration in an `EnvoyFilter` resource, which Istio can use to configure the proxy.

It defines a Custom Resource (CRD) that allows us to send a config to the set of proxies whose pods' labels match the selector:

```yaml
apiVersion: red-badger.com/v1alpha1
kind: FeatureTargetConfig
metadata:
  namespace: echo-service
  name: echo
spec:
  selector:
    app: echo
  configuration: |
    {
      "header_name": "x-features",
      "explicit": [
        {
          "_extract": "list",
          "attribute": "x-feature-override"
        },
        {
          "_extract": "pattern",
          "attribute": ":authority",
          "pattern": "f-*.localhost"
        }
      ]
    }
```

In this example, all pods that match the label selector will have their envoy side-cars configured with the specified configuration. This is managed by the operator and by Istio. If you update or delete the CRD, the relevant side-cars will take on, or remove, the configuration. You shouldn't need to restart any pods.

The above example will result in an `EnvoyFilter` that looks like this:

```yaml
apiVersion: networking.istio.io/v1alpha3
kind: EnvoyFilter
metadata:
  creationTimestamp: "2020-05-24T14:12:03Z"
  generation: 3
  labels:
    app.kubernetes.io/instance: ddf66595-45ff-4de5-8b15-f092aa05d1c7
    app.kubernetes.io/managed-by: feature-targeting
  name: echo-filter
  namespace: echo-service
  ownerReferences:
    - apiVersion: red-badger.com/v1alpha1
      controller: true
      kind: FeatureTargetConfig
      name: echo
      uid: ddf66595-45ff-4de5-8b15-f092aa05d1c7
  resourceVersion: "456699"
  selfLink: /apis/networking.istio.io/v1alpha3/namespaces/echo-service/envoyfilters/echo-filter
  uid: 3651c239-af91-4715-83c0-01fa3a1ace52
spec:
  configPatches:
    - applyTo: HTTP_FILTER
      match:
        context: SIDECAR_INBOUND
        listener:
          filterChain:
            filter:
              name: envoy.http_connection_manager
              subFilter:
                name: envoy.router
      patch:
        operation: INSERT_BEFORE
        value:
          name: envoy.filters.http.wasm
          typedConfig:
            "@type": type.googleapis.com/udpa.type.v1.TypedStruct
            typeUrl: type.googleapis.com/envoy.config.filter.http.wasm.v2.Wasm
            value:
              config:
                configuration: |
                  {
                    "header_name": "x-features",
                    "explicit": [
                      {
                        "_extract": "list",
                        "attribute": "x-feature-override"
                      },
                      {
                        "_extract": "pattern",
                        "attribute": ":authority",
                        "pattern": "f-*.localhost"
                      }
                    ]
                  }
                name: feature_targeting
                root_id: redbadger.feature_targeting
                vm_config:
                  allow_precompiled: true
                  code:
                    local:
                      filename: /var/local/lib/envoy-filters/feature_targeting.wasm
                  runtime: envoy.wasm.runtime.v8
                  vm_id: feature_targeting
  workloadSelector:
    labels:
      app: echo
```

You can also view the status of the CRD, in order to verify that there hasn't been a problem:

```sh
k describe features echo

Name:         echo
Namespace:    echo-service
Labels:       <none>
Annotations:  API Version:  red-badger.com/v1alpha1
Kind:         FeatureTargetConfig
Metadata:
  Creation Timestamp:  2020-05-24T13:15:31Z
  Finalizers:
    feature-targeting
  Generation:        4
  Resource Version:  456698
  Self Link:         /apis/red-badger.com/v1alpha1/namespaces/echo-service/featuretargetconfigs/echo
  UID:               ddf66595-45ff-4de5-8b15-f092aa05d1c7
Spec:
  Configuration:  {
  "header_name": "x-features",
  "explicit": [
    {
      "_extract": "list",
      "attribute": "x-feature-override"
    },
    {
      "_extract": "pattern",
      "attribute": ":authority",
      "pattern": "f-*.localhost"
    }
  ]
}

  Selector:
    App:  echo
Status:
  Message:              Filter created at: 2020-05-24T14:12:03Z
  Observed Generation:  4
  Phase:                Running
Events:                 <none>
```

## Installation and testing

Ensure your context points to a Kubernetes cluster running Istio 1.6+ and the [`adapter-proxy-wasm`](../adapter-proxy-wasm/README.md).

```sh
(cd manifests && make)
```

You can install the example CRD and test that the `x-features` header is populated when you curl the [echo service](../samples/echo-service/README.md).

```sh
(cd examples && make)

curl --resolve f-echo.localhost:80:127.0.0.1 -vvv http://f-echo.localhost -H x-feature-override:viktor

* Added f-echo.localhost:80:127.0.0.1 to DNS cache
* Hostname f-echo.localhost was found in DNS cache
*   Trying 127.0.0.1...
* TCP_NODELAY set
* Connected to f-echo.localhost (127.0.0.1) port 80 (#0)
> GET / HTTP/1.1
> Host: f-echo.localhost
> User-Agent: curl/7.64.1
> Accept: */*
> x-feature-override:viktor
>
< HTTP/1.1 200 OK
< x-powered-by: Express
< content-type: application/json; charset=utf-8
< content-length: 869
< etag: W/"365-YjtDHWFv65tp8bl4bntZeCTc2G4"
< date: Mon, 25 May 2020 08:42:14 GMT
< x-envoy-upstream-service-time: 5
< server: istio-envoy
<
* Connection #0 to host f-echo.localhost left intact
{"path":"/","headers":{"host":"f-echo.localhost","user-agent":"curl/7.64.1","accept":"*/*","x-feature-override":"viktor","x-forwarded-for":"192.168.65.3","x-forwarded-proto":"http","x-request-id":"42e52fee-c8e9-4c65-8fe5-11c9bd2bb05c","content-length":"0","x-envoy-internal":"true","x-forwarded-client-cert":"By=spiffe://cluster.local/ns/echo-service/sa/default;Hash=73ddec0d7911bd15c46bb2b7c38dbae1acefe7585de9aa2639b5ea4bccbc4a71;Subject=\"\";URI=spiffe://cluster.local/ns/istio-system/sa/istio-ingressgateway-service-account","x-features":"echo new_feature viktor","x-b3-traceid":"2c4fc919aebee1eb135e17fecd56a81c","x-b3-spanid":"064ca899e56f374d","x-b3-parentspanid":"135e17fecd56a81c","x-b3-sampled":"0"},"method":"GET","body":{},"fresh":false,"hostname":"f-echo.localhost","ip":"::ffff:127.0.0.1","ips":[],"protocol":"http","query":{},"subdomains":[],"xhr":false}* Closing connection 0
```
