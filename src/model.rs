use serde::{Deserialize, Serialize};

#[derive(Serialize)]
#[derive(Deserialize)]
#[derive(Clone)]
pub struct Model {
    id: String,
    object: String,
    created: u64,
    context_length: i32,
}

impl Model {
    pub fn new(&mut self, id: String, object: String, created: u64, context_length: i32) {
        self.id = id;
        self.object = object;
        self.created = created;
        self.context_length = context_length;
    }

    pub fn get_id(&self) -> &String {
        &self.id
    }

    pub fn get_object(&self) -> &String {
        &self.object
    }

    pub fn get_created(&self) -> &u64 {
        &self.created
    }

    pub fn get_context_length(&self) -> &i32 {
        &self.context_length
    }
}