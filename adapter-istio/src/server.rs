use prost::Message;
use tonic::{Code, Request, Response, Status};

pub use self::adapter_istio::handle_feature_targeting_service_server::HandleFeatureTargetingServiceServer;
use self::adapter_istio::{
    handle_feature_targeting_service_server::HandleFeatureTargetingService,
    HandleFeatureTargetingRequest, HandleFeatureTargetingResponse, OutputMsg, Params,
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

        let msg = request.into_inner();
        let config = msg
            .adapter_config
            .and_then(|cfg| Params::decode(cfg.value.as_ref()).ok())
            .and_then(|params| params.explicit_targeting)
            .map_or(features::ExplicitMatchingConfig::default(), |tgt| {
                features::ExplicitMatchingConfig {
                    host: tgt.hostname_pattern,
                    header: tgt.override_header,
                }
            });

        if let Some(inst) = msg.instance {
            let implicit_features = features::implicit(&inst);
            let explicit_features = features::explicit(&inst, &config);

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
