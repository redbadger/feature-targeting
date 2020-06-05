use data_plane::features;
use features::{explicit::Config as ExplicitConfig, implicit::Config as ImplicitConfig};
use log::{info, warn};
use proxy_wasm::{
    traits::*,
    types::{self, LogLevel},
};
use serde::Deserialize;
use std::{cell::RefCell, collections::HashMap};
use types::Action;

#[derive(Deserialize, Debug)]
struct FilterConfig {
    header_name: String,
    explicit: ExplicitConfig,
    implicit: ImplicitConfig,
}

impl Default for FilterConfig {
    fn default() -> Self {
        FilterConfig {
            header_name: "x-feature".to_owned(),
            explicit: ExplicitConfig::default(),
            implicit: ImplicitConfig::default(),
        }
    }
}

thread_local! {
    static CONFIGS: RefCell<HashMap<u32, FilterConfig>> = RefCell::new(HashMap::new())
}

#[no_mangle]
pub fn _start() {
    proxy_wasm::set_log_level(LogLevel::Trace);
    proxy_wasm::set_root_context(|context_id| -> Box<dyn RootContext> {
        CONFIGS.with(|configs| {
            configs
                .borrow_mut()
                .insert(context_id, FilterConfig::default());
        });

        Box::new(RootHandler { context_id })
    });
    proxy_wasm::set_http_context(|_context_id, root_context_id| -> Box<dyn HttpContext> {
        Box::new(HttpHandler { root_context_id })
    })
}

struct RootHandler {
    context_id: u32,
}

impl Context for RootHandler {}

impl RootContext for RootHandler {
    fn on_configure(&mut self, _config_size: usize) -> bool {
        let configuration: Vec<u8> = match self.get_configuration() {
            Some(c) => c,
            None => return false,
        };

        match serde_json::from_slice(configuration.as_ref()) {
            Ok(new_config) => {
                info!("Configuration changed: {:?}", new_config);
                CONFIGS.with(|configs| configs.borrow_mut().insert(self.context_id, new_config));

                true
            }
            Err(e) => {
                warn!("Error parsing configuration: {:?}", e);

                false
            }
        }
    }
}

struct HttpHandler {
    root_context_id: u32,
}

impl Context for HttpHandler {}

impl HttpContext for HttpHandler {
    fn on_http_request_headers(&mut self, _num_headers: usize) -> Action {
        CONFIGS.with(|configs| {
            if let Some(config) = configs.borrow().get(&self.root_context_id) {
                let mut request: HashMap<&str, &str> = HashMap::new();

                let headers = self.get_http_request_headers();
                for (name, value) in &headers {
                    request.insert(name.as_ref(), value.as_ref());
                }

                info!(
                    "Targeting on request: {:?}, with configuration: {:?}",
                    request, config
                );
                let output = features::target(&request, &config.explicit, &config.implicit);
                self.set_http_request_header(config.header_name.as_ref(), Some(output.as_ref()));

                Action::Continue
            } else {
                warn!(
                    "Configuration does not exist for root context #{}, this should not happen!",
                    self.root_context_id
                );

                Action::Continue
            }
        })
    }
}
