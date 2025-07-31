use crate::errors::DarwinV7Error;
use crate::team::Team;
use serde_yaml;
use std::io::Read;
use std::path::Path;
use std::{collections::HashMap, fs::File};

#[derive(Debug, Clone)]
pub struct Config {
    base_url: String,
    api_endpoint: String,
    default_team: String,
    teams: HashMap<String, Team>,
}

impl Config {
    pub fn new(
        base_url: String,
        api_endpoint: String,
        default_team: String,
        teams: HashMap<String, Team>,
    ) -> Self {
        Self {
            base_url,
            api_endpoint,
            default_team,
            teams,
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn api_endpoint(&self) -> &str {
        &self.api_endpoint
    }

    pub fn default_team(&self) -> &String {
        &self.default_team
    }

    pub fn teams(&self) -> &HashMap<String, Team> {
        &self.teams
    }

    pub fn from_file<T>(file_path: T) -> Result<Self, DarwinV7Error>
    where
        T: AsRef<Path>,
    {
        let mut file = File::open(&file_path).map_err(|_| {
            DarwinV7Error::InvalidConfigError("Unable to read config file.".to_string())
        })?;
        let mut buffer = String::new();
        file.read_to_string(&mut buffer).map_err(|_| {
            DarwinV7Error::InvalidConfigError("Unable to read config file.".to_string())
        })?;

        Self::try_from(buffer.as_str())
    }
}

fn get_from_yaml<'b>(value: &'b serde_yaml::Value, key: &'b str) -> Result<&'b str, DarwinV7Error> {
    value
        .get(key)
        .ok_or(DarwinV7Error::InvalidConfigError(format!(
            "Missing '{key}' from config"
        )))?
        .as_str()
        .ok_or(DarwinV7Error::InvalidConfigError(format!(
            "{key} cannot be represented as a string"
        )))
}

impl TryFrom<&str> for Config {
    type Error = DarwinV7Error;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let config_value: serde_yaml::Value = serde_yaml::from_str(value).map_err(|_| {
            DarwinV7Error::InvalidConfigError("Unable to parse config file".to_string())
        })?;

        let key = "global";
        let global = config_value
            .get(key)
            .ok_or(DarwinV7Error::InvalidConfigError(format!(
                "Missing '{key}' from config"
            )))?;

        let base_url = get_from_yaml(global, "base_url")?.to_string();
        let api_endpoint = get_from_yaml(global, "api_endpoint")?.to_string();
        let default_team = get_from_yaml(global, "default_team")?.to_string();

        let mut teams: HashMap<String, Team> = HashMap::new();

        let key = "teams";
        let team_mapping = config_value
            .get(key)
            .ok_or(DarwinV7Error::InvalidConfigError(format!(
                "Missing '{key}' from config"
            )))?
            .as_mapping()
            .ok_or(DarwinV7Error::InvalidConfigError(format!(
                "Missing '{key}' from config"
            )))?;

        for team in team_mapping.iter() {
            let team = Team::try_from(team)?;
            teams.insert(team.slug.to_string(), team);
        }

        Ok(Self {
            base_url,
            api_endpoint,
            default_team,
            teams,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    const CONFIG_STR: &str = "global:
  api_endpoint: https://darwin.v7labs.com/api/
  base_url: https://darwin.v7labs.com
  default_team: team-a
teams:
  team-a:
    api_key: 1ed99664-726e-4400-bc5d-3132b22ce60c
    datasets_dir: /home/user/.v7/team-a
  team-b:
    api_key: b5509922-38e4-4ff9-b976-fbb42c077e45
    datasets_dir: /home/user/.v7/team-b
";

    #[test]
    fn test_config_from_str() {
        let config = Config::try_from(CONFIG_STR).unwrap();

        assert_eq!(config.api_endpoint(), "https://darwin.v7labs.com/api/");
        assert_eq!(config.base_url(), "https://darwin.v7labs.com");

        // Get the known default team
        let default_team = config.teams().get(config.default_team()).unwrap();
        assert_eq!(default_team.slug, "team-a");

        // Check the teams
        let team_keys: Vec<String> = config.teams().keys().cloned().collect();
        assert_eq!(team_keys.len(), 2);
        assert!(team_keys.contains(&"team-a".to_string()));
        assert!(team_keys.contains(&"team-b".to_string()));

        // Check the default team
        let default_team = config.teams().get(config.default_team()).unwrap();
        assert_eq!(default_team.slug, "team-a");

        // Get the other team
        let other_team = config.teams().get("team-b").unwrap();
        assert_eq!(
            other_team.api_key.as_ref().unwrap(),
            "b5509922-38e4-4ff9-b976-fbb42c077e45"
        );
    }

    #[test]
    fn test_config_from_file() {
        // Write the file out
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{}", CONFIG_STR).unwrap();

        let config = Config::from_file(file.path()).unwrap();

        assert_eq!(config.api_endpoint(), "https://darwin.v7labs.com/api/");
        assert_eq!(config.base_url(), "https://darwin.v7labs.com");

        // Get the known default team
        let default_team = config.teams().get(config.default_team()).unwrap();
        assert_eq!(default_team.slug, "team-a");
    }
}
