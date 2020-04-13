use log::info;
use std::collections::HashMap;

use data_plane::features;
use proxy_wasm as wasm;

#[no_mangle]
pub fn _start() {
    proxy_wasm::set_log_level(wasm::types::LogLevel::Trace);
    proxy_wasm::set_http_context(
        |context_id, _root_context_id| -> Box<dyn wasm::traits::HttpContext> {
            Box::new(FeatureTargeting {
                context_id,
                // TODO use on_configure to allow operator to specify this
                explicit_matching_config: features::ExplicitMatchingConfig {
                    header: "x-feature-override".to_owned(),
                    host: "f-*.localhost".to_owned(),
                },
            })
        },
    )
}

struct FeatureTargeting {
    context_id: u32,
    explicit_matching_config: features::ExplicitMatchingConfig,
}

impl wasm::traits::Context for FeatureTargeting {}

impl wasm::traits::HttpContext for FeatureTargeting {
    fn on_http_request_headers(&mut self, num_headers: usize) -> wasm::types::Action {
        info!(
            "Got {} HTTP headers in #{}. Using config: {:?}",
            num_headers, self.context_id, self.explicit_matching_config
        );
        let mut request: HashMap<&str, &str> = HashMap::new();
        let headers = self.get_http_request_headers();

        for (name, value) in &headers {
            request.insert(name.as_ref(), value.as_ref());

            // TODO improve explicit matching config to allow list and pattern
            // matches on any request attributes
            if name == ":authority" {
                // in Envoy, :authority is requested "host:port"
                if let Some(host) = value.split(':').next() {
                    request.insert("host", host);
                }
            }
        }

        info!("Targeting on request: {:?}", request);

        let exp_features = features::explicit(&request, &self.explicit_matching_config);
        let imp_features = features::implicit(&request);

        let output = features::union(exp_features.as_ref(), imp_features.as_ref());

        // TODO Expose as configuration
        self.set_http_request_header("x-features", Some(output.as_ref()));

        wasm::types::Action::Continue
    }
}
