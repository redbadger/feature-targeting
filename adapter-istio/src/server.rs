pub use self::adapter_istio::handle_feature_targeting_service_server::HandleFeatureTargetingServiceServer;
use self::adapter_istio::{
    handle_feature_targeting_service_server::HandleFeatureTargetingService,
    HandleFeatureTargetingRequest, HandleFeatureTargetingResponse, OutputMsg, Params,
};
use data_plane::features;
use features::explicit;
use istio::mixer::adapter::model::v1beta1::CheckResult;
use prost::Message;
use std::collections::HashMap;
use tonic::{Code, Request, Response, Status};

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
            .map_or(explicit::Config::default(), |tgt| {
                explicit::Config(vec![
                    explicit::Extract::List(explicit::List {
                        attribute: tgt.override_header,
                    }),
                    explicit::Extract::Pattern(explicit::Pattern {
                        attribute: "host".to_owned(),
                        pattern: tgt.hostname_pattern,
                    }),
                ])
            });
        println!("{:?}", config);

        if let Some(inst) = msg.instance {
            let mut request: HashMap<&str, &str> = inst
                .headers
                .iter()
                .map(|(k, v)| (k.as_ref(), v.as_ref()))
                .collect();
            request.insert("method", inst.method.as_ref());
            request.insert("path", inst.path.as_ref());

            let ftrs = features::target(&request, &config);

            let reply = HandleFeatureTargetingResponse {
                output: Some(OutputMsg { features: ftrs }),
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
