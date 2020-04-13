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
                explicit_matching_config: features::explicit::Config(vec![
                    Box::new(features::explicit::List {
                        attribute: "x-feature-override".to_owned(),
                    }),
                    Box::new(features::explicit::Pattern {
                        attribute: ":authority".to_owned(),
                        pattern: "f-*.localhost:8080".to_owned(),
                    }),
                ]),
            })
        },
    )
}

struct FeatureTargeting {
    context_id: u32,
    explicit_matching_config: features::explicit::Config,
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
        }

        info!("Targeting on request: {:?}", request);

        let output = features::target(&request, &self.explicit_matching_config);

        // TODO Expose as configuration
        self.set_http_request_header("x-features", Some(output.as_ref()));

        wasm::types::Action::Continue
    }
}
