use tonic::{Code, Request, Response, Status};

pub use self::adapter_istio::handle_feature_targeting_service_server::HandleFeatureTargetingServiceServer;
use self::adapter_istio::{
    handle_feature_targeting_service_server::HandleFeatureTargetingService,
    HandleFeatureTargetingRequest, HandleFeatureTargetingResponse, OutputMsg,
};
use crate::features;
use istio::mixer::adapter::model::v1beta1::CheckResult;

pub mod adapter_istio {
    tonic::include_proto!("featuretargeting");
}
pub mod istio {
    pub mod mixer {
        pub mod adapter {
            pub mod model {
                pub mod v1beta1 {
                    tonic::include_proto!("istio.mixer.adapter.model.v1beta1");
                }
            }
        }
    }
}
pub mod google {
    pub mod protobuf {
        tonic::include_proto!("google.protobuf");
    }
    pub mod rpc {
        tonic::include_proto!("google.rpc");
    }
}

#[derive(Debug, Default)]
pub struct Service {}

#[tonic::async_trait]
impl HandleFeatureTargetingService for Service {
    async fn handle_feature_targeting(
        &self,
        request: Request<HandleFeatureTargetingRequest>,
    ) -> Result<Response<HandleFeatureTargetingResponse>, Status> {
        println!("{:?}", request);

        if let Some(msg) = request.into_inner().instance {
            let requested_features = match msg.features.get("requested") {
                Some(s) => s.to_string(),
                None => String::new(),
            };
            let reply = HandleFeatureTargetingResponse {
                output: Some(OutputMsg {
                    features: features::add_features(requested_features, &["new_feature"]),
                }),
                result: None::<CheckResult>,
            };
            Ok(Response::new(reply))
        } else {
            Err(Status::new(
                Code::InvalidArgument,
                "Request had no instance",
            ))
        }
    }
}
