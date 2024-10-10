use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Configuration {
    pub services: Vec<Service>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Service {
    pub name: String,
    pub prefix: String,
    pub sources: Vec<Source>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Source {
    pub endpoint: String,
    pub method: String,
    pub content_type: String,
    pub response: String,
    pub status: u16,
    pub query_params: Option<Vec<String>>
}

impl Configuration {
    pub async fn read_config(file_path: String) -> Result<Configuration, ()> {
        let open_file = &std::fs::read_to_string(&file_path).unwrap();

        let conf = serde_yaml::from_str::<Configuration>(open_file)
            .unwrap();

        Ok(conf)
    }
}