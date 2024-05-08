use std::fs::File;
use std::io::Read;
use crate::model::Model;

static mut MODELS: Vec<Model> = Vec::new();

pub unsafe fn load_models() -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::open("models.json")?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let models: Vec<Model> = serde_json::from_str(&contents)?;

    MODELS = models;

    Ok(())
}

pub unsafe fn get_models() -> Vec<Model> {
    MODELS.clone()
}