mod compute;
mod random;
#[cfg(target_arch = "wasm32")]
mod render;
#[cfg(target_arch = "wasm32")]
mod state;
mod fft_compute;
mod uniforms_manager;
mod storage_manager;

#[cfg(target_arch = "wasm32")]
pub use wasm_interface::*;

#[cfg(target_arch = "wasm32")]
mod wasm_interface {

    use crate::state::State;
    use wasm_bindgen::prelude::*;

    #[derive(serde::Deserialize, serde::Serialize)]
    pub struct Parameters {
        pub random_seed: u32,
        pub random_density: f32,
        pub random_brush_size: u32,
        pub compute_time_step: u32,
        pub compute_m: f32,
        pub compute_s: f32,
    }


    #[wasm_bindgen(typescript_custom_section)]
    const PARAMETERS_TS: &'static str = r#"
        type Parameters = {
            random_seed: number,
            random_density: number,
            random_brush_size: number,
            compute_time_step: number,
            compute_m: number,
            compute_s: number,
        }
    "#;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(typescript_type = "ParametersTs")]
        #[derive(Debug)]
        pub type ParametersTs;
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
        pub fn step(&mut self) {
            self.state.step();
        }

        #[wasm_bindgen]
        pub fn render_frame(&mut self) {
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
        pub fn set_parameters(&mut self, parameters: ParametersTs) {
            let parameters = serde_wasm_bindgen::from_value(parameters.dyn_into::<JsValue>().unwrap()).unwrap();
            self.state.parse_parameters(parameters);
        }
    }

}