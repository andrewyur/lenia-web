mod compute;
mod render;
mod state;

use state::State;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct App {
    state: State,
}

#[wasm_bindgen]
impl App {
    #[wasm_bindgen]
    pub async fn new(canvas: web_sys::HtmlCanvasElement) -> Result<Self, JsValue> {
        console_error_panic_hook::set_once();
        console_log::init_with_level(log::Level::Debug).unwrap();

        let state = State::new(canvas)
            .await
            .map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;

        Ok(Self { state })
    }

    #[wasm_bindgen]
    pub fn render_frame(&mut self, step: bool) -> Result<(), JsValue> {
        self.state.update();
        if step {
            self.state.step().map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
        }
        self.state
            .render()
            .map_err(|e| JsValue::from_str(&format!("{:?}", e)))
    }

    #[wasm_bindgen]
    pub fn handle_resize(&mut self, width: u32, height: u32) {
        self.state.resize(width, height);
    }
}
