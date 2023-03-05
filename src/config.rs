use std::env;

pub const NOTION_API_TOKEN: &str = "NOTION_API_TOKEN";
pub const NOTION_SOURCE_DATABASE_ID: &str = "NOTION_SOURCE_DATABASE_ID";
pub const NOTION_FEED_DATABASE_ID: &str = "NOTION_FEED_DATABASE_ID";

#[derive(Debug)]
pub struct Config {
    pub notion_api_token: String,
    pub notion_source_database_id: String,
    pub notion_feed_database_id: String,
}

impl Config {
    pub fn new(
        notion_api_token: Option<String>,
        notion_source_database_id: Option<String>,
        notion_feed_database_id: Option<String>,
    ) -> Result<Config, String> {
        let notion_api_token = get_config_value(notion_api_token, NOTION_API_TOKEN)?;
        let notion_source_database_id =
            get_config_value(notion_source_database_id, NOTION_SOURCE_DATABASE_ID)?;
        let notion_feed_database_id =
            get_config_value(notion_feed_database_id, NOTION_FEED_DATABASE_ID)?;

        Ok(Self {
            notion_api_token,
            notion_source_database_id,
            notion_feed_database_id,
        })
    }
}

fn get_config_value(name: Option<String>, env_name: &str) -> Result<String, String> {
    if let Some(name) = name {
        if !name.is_empty() {
            return Ok(name);
        }
        return Err(format!("Invalid config variable: {name:?}"));
    }

    let env_var = env::var(env_name);

    if let Ok(ref env_var) = env_var {
        if !env_var.is_empty() {
            return Ok(env_var.to_string());
        }
    }

    return Err(format!("Invalid config variable: {env_var:?}"));
}
