pub mod config {

    use serde::{Deserialize, Serialize};
    use toml;

    #[derive(Serialize, Deserialize)]
    pub struct Config {
        pub omdb_key: Option<String>,
        pub movie_folders: Vec<String>,
    }

    const CONFIG_PATH: &str = ".config/movieViewer/movieViewer.toml";
    type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

    pub fn load_config(user_relative_path: &str) -> Result<(std::ffi::OsString, String)> {
        let home_dir = std::env::var_os("HOME").ok_or("no home directory")?;
        let mut config_path = std::path::PathBuf::new();
        config_path.push(home_dir);
        config_path.push(user_relative_path);
        let config_content = if let Ok(content) = std::fs::read(&config_path) {
            String::from_utf8_lossy(&content).to_string()
        } else {
            "This is the default content\n".to_owned()
        };
        Ok((config_path.into_os_string(), config_content))
    }

    pub fn save_config(config_path: &std::ffi::OsStr, config_content: &str) -> Result<()> {
        let dir_name = std::path::Path::new(&config_path)
            .parent()
            .ok_or("incorrect directory")?;
        std::fs::create_dir_all(dir_name)?;
        std::fs::write(&config_path, config_content)?;
        Ok(())
    }

    pub fn get_or_create_config() -> Result<(Config)> {
        let config = load_config(&CONFIG_PATH);
        match config {
            Ok((path, content)) => {
                let parsed_config: Config = toml::from_str(&content)?;
                Ok(parsed_config)
            }
            Err(error) => {
                let base_config = Config {
                    omdb_key: None,
                    movie_folders: vec![],
                };
                let saved = save_config(
                    &std::ffi::OsString::from(&CONFIG_PATH),
                    &toml::to_string(&base_config).unwrap(),
                );
                match saved {
                    Ok(_) => Ok(base_config),
                    Err(error) => Err(error),
                }
            }
        }
    }
}
