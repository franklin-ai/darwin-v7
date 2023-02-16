use crate::types::Team;
use anyhow::{Context, Result};
use serde_yaml;
use std::io::Read;
use std::path::Path;
use std::{collections::HashMap, fs::File, path::PathBuf};

pub const DEFAULT_CONFIG_PATH: &str = "~/.darwin/config";

pub struct Config {
    path: Option<PathBuf>,
    base_url: String,
    api_endpoint: String,
    default_team: String,
    teams: HashMap<String, Team>,
}

impl Config {
    pub fn path(&self) -> &Option<PathBuf> {
        &self.path
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn api_endpoint(&self) -> &str {
        &self.api_endpoint
    }

    pub fn default_team(&self) -> &Team {
        self.teams.get(&self.default_team).unwrap()
    }

    pub fn teams(&self) -> &HashMap<String, Team> {
        &self.teams
    }

    pub fn from_file<T: AsRef<Path>>(file_path: T) -> Result<Self> {
        let mut file = File::open(file_path)?;
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)?;

        Self::try_from(buffer.as_str())
    }
}

fn get_from_yaml<'b>(value: &'b serde_yaml::Value, key: &'b str) -> anyhow::Result<&'b str> {
    value
        .get(key)
        .context("Missing '{key}' from config")?
        .as_str()
        .context("{key} cannot be represented as a string")
}

impl TryFrom<&str> for Config {
    type Error = anyhow::Error;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let config_value: serde_yaml::Value = serde_yaml::from_str(value)?;

        let global = config_value
            .get("global")
            .context("Missing 'global' map from config")?;

        let base_url = get_from_yaml(global, "base_url")?.to_string();
        let api_endpoint = get_from_yaml(global, "api_endpoint")?.to_string();
        let default_team = get_from_yaml(global, "default_team")?.to_string();

        let mut teams: HashMap<String, Team> = HashMap::new();

        let team_mapping = config_value
            .get("teams")
            .context("Missing 'teams' map from config")?
            .as_mapping()
            .context("'teams' not correctly defined")?;

        for team in team_mapping.iter() {
            let team = Team::try_from(team)?;
            teams.insert(team.slug().to_string(), team);
        }

        Ok(Self {
            path: None, // If provided from a string is set to None
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

    const CONFIG_STR: &'static str = "global:
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
        let config: Config = CONFIG_STR.try_into().unwrap();

        assert_eq!(config.api_endpoint(), "https://darwin.v7labs.com/api/");
        assert_eq!(config.base_url(), "https://darwin.v7labs.com");

        // Get the known default team
        let default_team = config.teams().get("team-a").unwrap();
        assert_eq!(config.default_team(), default_team);

        // Check the teams
        let team_keys: Vec<String> = config.teams().keys().map(|x| x.clone()).collect();
        assert_eq!(team_keys.len(), 2);
        assert!(team_keys.contains(&"team-a".to_string()));
        assert!(team_keys.contains(&"team-b".to_string()));

        // Check the default team
        let default_team = config.default_team();
        assert_eq!(default_team.slug(), "team-a");

        // Get the other team
        let other_team = config.teams().get("team-b").unwrap();
        assert_eq!(
            other_team.api_key().as_ref().unwrap(),
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
        let default_team = config.teams().get("team-a").unwrap();
        assert_eq!(config.default_team(), default_team);
    }
}
