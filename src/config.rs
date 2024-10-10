use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Configuration {
    pub service: Service,
    pub response: Response
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Service {
    pub name: String,
    pub version: String,
    pub host: String,
    pub port: String
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Response {
    pub config_file: String
}

impl Configuration {
    pub async fn read_config_file(file_path: String) -> Result<Configuration, ()> {
        let open_file = &std::fs::read_to_string(&file_path).unwrap();

        let conf = serde_yaml::from_str::<Configuration>(open_file)
            .unwrap();

        Ok(conf)
    }
}