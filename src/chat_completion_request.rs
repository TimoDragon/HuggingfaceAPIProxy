use serde::Deserialize;

#[derive(Deserialize)]
pub struct ChatCompletionRequest {
    model: String
}

impl ChatCompletionRequest {
    pub fn get_model(&self) -> &String {
        &self.model
    }
}