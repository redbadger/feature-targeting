use log::{info, warn};
use std::cell::RefCell;
use std::collections::HashMap;

use features::explicit::Config as ExplicitConfig;

use data_plane::features;
use proxy_wasm as wasm;

#[derive(Debug)]
struct FilterConfig {
    header_name: String,
    explicit: ExplicitConfig,
}

impl Default for FilterConfig {
    fn default() -> Self {
        FilterConfig {
            header_name: "x-feature".to_owned(),
            explicit: ExplicitConfig::default(),
        }
    }
}

thread_local! {
    static CONFIGS: RefCell<HashMap<u32, FilterConfig>> = RefCell::new(HashMap::new())
}

#[no_mangle]
pub fn _start() {
    proxy_wasm::set_log_level(wasm::types::LogLevel::Trace);
    proxy_wasm::set_root_context(|context_id| -> Box<dyn wasm::traits::RootContext> {
        CONFIGS.with(|configs| {
            configs
                .borrow_mut()
                .insert(context_id, FilterConfig::default());
        });

        Box::new(RootHandler { context_id })
    });
    proxy_wasm::set_http_context(
        |context_id, root_context_id| -> Box<dyn wasm::traits::HttpContext> {
            Box::new(HttpHandler {
                root_context_id,
                context_id,
            })
        },
    )
}

struct RootHandler {
    context_id: u32,
}

impl wasm::traits::Context for RootHandler {}

impl wasm::traits::RootContext for RootHandler {
    fn on_configure(&mut self, _config_size: usize) -> bool {
        info!("#{} configure!", self.context_id,);

        let configuration = self.get_configuration();
        info!("Configuration: {:?}", configuration);

        let new_config = FilterConfig {
            header_name: "x-features".to_owned(),
            explicit: features::explicit::Config(vec![
                Box::new(features::explicit::List {
                    attribute: "x-feature-override".to_owned(),
                }),
                Box::new(features::explicit::Pattern {
                    attribute: ":authority".to_owned(),
                    pattern: "f-*.localhost:8080".to_owned(),
                }),
            ]),
        };

        CONFIGS.with(|configs| configs.borrow_mut().insert(self.context_id, new_config));

        true
    }
}

struct HttpHandler {
    root_context_id: u32,
    context_id: u32,
}

impl wasm::traits::Context for HttpHandler {}

impl wasm::traits::HttpContext for HttpHandler {
    fn on_http_request_headers(&mut self, num_headers: usize) -> wasm::types::Action {
        CONFIGS.with(|configs| {
            if let Some(config) = configs.borrow().get(&self.root_context_id) {
                info!(
                    "Got {} HTTP headers in #{}/#{}. Using config: {:?}",
                    num_headers, self.root_context_id, self.context_id, config
                );

                let mut request: HashMap<&str, &str> = HashMap::new();

                let headers = self.get_http_request_headers();
                for (name, value) in &headers {
                    request.insert(name.as_ref(), value.as_ref());
                }

                info!("Targeting on request: {:?}", request);
                let output = features::target(&request, &config.explicit);
                self.set_http_request_header(config.header_name.as_ref(), Some(output.as_ref()));

                wasm::types::Action::Continue
            } else {
                warn!(
                    "Config does not exist for root context #{}, this should not happen!",
                    self.root_context_id
                );

                wasm::types::Action::Continue
            }
        })
    }
}
