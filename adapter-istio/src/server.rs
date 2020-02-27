use regex::Regex;
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

#[derive(Debug)]
pub struct Service {
    explicit_matching_config: features::ExplicitMatchingConfig,
}

impl Service {
    pub fn new(host_matching: Regex, header_name: &str) -> Service {
        Service {
            explicit_matching_config: features::ExplicitMatchingConfig {
                host: host_matching,
                header: header_name.to_owned(),
            },
        }
    }
}

#[tonic::async_trait]
impl HandleFeatureTargetingService for Service {
    async fn handle_feature_targeting(
        &self,
        request: Request<HandleFeatureTargetingRequest>,
    ) -> Result<Response<HandleFeatureTargetingResponse>, Status> {
        println!("{:?}", request);

        if let Some(msg) = request.into_inner().instance {
            let implicit_features = features::implicit(&msg);
            let explicit_features = features::explicit(&msg, &self.explicit_matching_config);

            let reply = HandleFeatureTargetingResponse {
                output: Some(OutputMsg {
                    features: features::union(
                        explicit_features.as_ref(),
                        implicit_features.as_ref(),
                    ),
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
