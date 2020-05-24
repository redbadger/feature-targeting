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

    let operator_config = OperatorConfig::new(OPERATOR_NAME, PARENT_TYPE);

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
        "message": "Ok", // TODO
        "phase": "Running",
    });
    Ok(SyncResponse {
        status,
        children: vec![],
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
