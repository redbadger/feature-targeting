//! Example of using roperator to create an operator for an `FeatureTargetConfig` example Custom Resource
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;

use anyhow::anyhow;
use log::{error, info};
use roperator::prelude::*;
use serde_json::value::Value;
use std::{collections::BTreeMap, time::Duration};

/// Name of our operator, which is automatically added as a label value in all of the child resources we create
const OPERATOR_NAME: &str = "feature-targeting";

/// a `K8sType` with basic info about our parent CRD
static PARENT_TYPE: &K8sType = &K8sType {
    api_version: "red-badger.com/v1alpha1",
    kind: "FeatureTargetConfig",
    plural_kind: "featuretargetconfigs",
};

static ENVOY_FILTER_TYPE: &K8sType = &K8sType {
    api_version: "networking.istio.io/v1alpha3",
    kind: "EnvoyFilter",
    plural_kind: "envoyfilters",
};

/// Represents an instance of the CRD that is in the kubernetes cluster.
/// Note that this struct does not need to implement Serialize because the
/// operator will only ever update the `status` sub-resource
#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct FeatureTargetConfig {
    pub metadata: Metadata,
    pub spec: FeatureTargetSpec,
    pub status: Option<FeatureTargetStatus>,
}

/// defines only the fields we care about from the metadata. We could also just use the `ObjectMeta` struct from the `k8s_openapi` crate.
#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct Metadata {
    pub namespace: String,
    pub name: String,
}

/// The spec of our CRD
#[derive(Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FeatureTargetSpec {
    pub selector: Option<BTreeMap<String, String>>,
    pub configuration: String,
}

/// Represents the status of a parent FeatureTargetConfig instance
#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct FeatureTargetStatus {
    pub message: String,
}

pub fn start() -> anyhow::Result<()> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "roperator=info,warn");
    }
    env_logger::init();

    let operator_config = OperatorConfig::new(OPERATOR_NAME, PARENT_TYPE)
        .with_child(ENVOY_FILTER_TYPE, ChildConfig::replace());

    Err(anyhow!(
        "error running operator: {}",
        roperator::runner::run_operator(operator_config, (handle_sync, handle_error),)
    ))
}

/// This function will invoked by the operator any time there's a change to any parent or child resources.
/// This just needs to return the desired parent status as well as the desired state for any children.
fn handle_sync(request: &SyncRequest) -> Result<SyncResponse, Error> {
    info!("Got sync request: {:?}", request);

    let status = json!({
        "message": get_current_status_message(request),
        "phase": "Running",
    });

    let children = get_desired_children(request)?;
    Ok(SyncResponse {
        status,
        children,
        resync: None,
    })
}

/// This function gets called by the operator whenever the sync handler responds with an error.
/// It needs to respond with the appropriate status for the given request and error and the minimum length of
/// time to wait before calling `handle_sync` again.
fn handle_error(request: &SyncRequest, err: Error) -> (Value, Option<Duration>) {
    error!("Failed to process request: {:?}\nCause: {:?}", request, err);

    let status = json!({
        "message": err.to_string(),
        "phase": "Error",
    });

    (status, None)
}

fn get_current_status_message(request: &SyncRequest) -> String {
    request
        .children()
        .of_type(ENVOY_FILTER_TYPE)
        .first()
        .map(|p| {
            format!(
                "Filter created at: {}",
                p.pointer("/metadata/creationTimestamp")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown")
            )
        })
        .unwrap_or_else(|| "Waiting for Filter to be initialized".to_owned())
}

fn get_desired_children(request: &SyncRequest) -> Result<Vec<Value>, Error> {
    let custom_resource: FeatureTargetConfig = request.deserialize_parent()?;
    let configuration = custom_resource.spec.configuration.as_str();
    let selector = custom_resource.spec.selector;

    let name = format!("{}-filter", custom_resource.metadata.name);
    let namespace = custom_resource.metadata.namespace.as_str();

    let filter = json!({
      "apiVersion": ENVOY_FILTER_TYPE.api_version,
      "kind": ENVOY_FILTER_TYPE.kind,
      "metadata": {
        "name": name,
        "namespace": namespace,
      },
      "spec": {
        "workloadSelector": {
          "labels": selector,
        },
        "configPatches": [
          {
            "applyTo": "HTTP_FILTER",
            "match": {
              "context": "SIDECAR_INBOUND",
              "listener": {
                "filterChain": {
                  "filter": {
                    "name": "envoy.http_connection_manager",
                    "subFilter": {
                      "name": "envoy.router",
                    }
                  }
                }
              }
            },
            "patch": {
              "operation": "INSERT_BEFORE",
              "value": {
                "name": "envoy.filters.http.wasm",
                "typedConfig": {
                  "@type": "type.googleapis.com/udpa.type.v1.TypedStruct",
                  "typeUrl": "type.googleapis.com/envoy.config.filter.http.wasm.v2.Wasm",
                  "value": {
                    "config": {
                      "name": "feature_targeting",
                      "configuration": json!(configuration),
                      "root_id": "redbadger.feature_targeting",
                      "vm_config": {
                        "code": {
                          "local": {
                            "filename": "/var/local/lib/envoy-filters/feature_targeting.wasm"
                          }
                        },
                        "runtime": "envoy.wasm.runtime.v8",
                        "vm_id": "feature_targeting",
                        "allow_precompiled": true,
                      }
                    }
                  }
                }
              }
            }
          }
        ]
      }
    });

    Ok(vec![filter])
}
