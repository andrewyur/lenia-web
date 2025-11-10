mod compute;
mod random;
mod render;
mod state;

use state::State;
use wasm_bindgen::prelude::*;

use crate::{compute::{ComputeConfig, ComputeUniforms}, random::{RandomConfig, RandomUniforms}};


#[wasm_bindgen]
#[derive(serde::Deserialize, serde::Serialize, Default)]
struct AppUniforms {
    random: RandomUniforms,
    compute: ComputeUniforms
}

#[wasm_bindgen(typescript_custom_section)]
const APP_CONFIG: &'static str = r#"
    type AppConfig = {
        random: RandomConfig,
        compute: ComputeConfig,
    }
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "AppConfig")]
    pub type AppConfig;
}

#[wasm_bindgen]
pub struct App {
    state: State,
}

#[wasm_bindgen]
impl App {
    #[wasm_bindgen]
    pub async fn new(canvas: web_sys::HtmlCanvasElement) -> Self {
        console_error_panic_hook::set_once();
        if let Err(e) = console_log::init_with_level(log::Level::Debug) {
            web_sys::console::log_1(&JsValue::from_str(&format!("Could not set rust log level: {:?}", e)));
        }

        let state = State::new(canvas).await.unwrap();

        Self { state }
    }

    #[wasm_bindgen]
    pub fn render_frame(&mut self, step: bool) {
        self.state.update();
        if step {
            self.state.step()
        }
        self.state.render();
    }

    #[wasm_bindgen]
    pub fn handle_resize(&mut self, width: u32, height: u32) {
        self.state.resize(width, height);
    }

    #[wasm_bindgen]
    pub fn randomize(&mut self, x: u32, y: u32) {
        self.state.randomize_area(x, y);
    }

    #[wasm_bindgen]
    pub fn clear(&mut self) {
        self.state.clear();
    }

    #[wasm_bindgen]
    pub fn default_config() -> AppConfig {
        serde_wasm_bindgen::to_value(&AppUniforms::default()).unwrap().dyn_into::<AppConfig>().unwrap()
    }

    #[wasm_bindgen]
    pub fn set_random_values(&self, uniforms_obj: RandomConfig) {
        let uniforms = serde_wasm_bindgen::from_value(uniforms_obj.dyn_into::<JsValue>().unwrap()).unwrap();
        self.state.write_random_uniforms(uniforms);
    }

    #[wasm_bindgen]
    pub fn set_compute_values(&self, uniforms_obj: ComputeConfig) {
        let uniforms = serde_wasm_bindgen::from_value(uniforms_obj.dyn_into::<JsValue>().unwrap()).unwrap();
        self.state.write_compute_uniforms(uniforms);
    }
}
