use log::info;

use proxy_wasm as wasm;

#[no_mangle]
pub fn _start() {
    proxy_wasm::set_log_level(wasm::types::LogLevel::Trace);
    proxy_wasm::set_http_context(
        |context_id, _root_context_id| -> Box<dyn wasm::traits::HttpContext> {
            Box::new(FeatureTargeting { context_id })
        },
    )
}

struct FeatureTargeting {
    context_id: u32,
}

impl wasm::traits::Context for FeatureTargeting {}

impl wasm::traits::HttpContext for FeatureTargeting {
    fn on_http_request_headers(&mut self, num_headers: usize) -> wasm::types::Action {
        info!("Got {} HTTP headers in #{}", num_headers, self.context_id);

        for (name, value) in &self.get_http_request_headers() {
            if name == "x-features" {
                info!("#{} -> {}: {}", self.context_id, name, value);
            }
        }

        wasm::types::Action::Continue
    }

    fn on_log(&mut self) {
        info!("#{} completed.", self.context_id);
    }
}
