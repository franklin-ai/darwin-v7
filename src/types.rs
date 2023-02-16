use anyhow::Context;
use serde_yaml::Value;
use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Team {
    slug: String,
    datasets_dir: Option<PathBuf>,
    api_key: Option<String>,
}

impl Team {
    pub fn slug(&self) -> &str {
        &self.slug
    }

    pub fn datasets_dir(&self) -> &Option<PathBuf> {
        &self.datasets_dir
    }

    pub fn api_key(&self) -> &Option<String> {
        &self.api_key
    }
}

impl TryFrom<(&Value, &Value)> for Team {
    type Error = anyhow::Error;

    fn try_from(value: (&Value, &Value)) -> Result<Self, Self::Error> {
        let slug = value.0.as_str().context("Invalid team slug")?.to_string();
        let api_key: Option<String> = match value.1.get("api_key") {
            Some(key) => Some(key.as_str().context("Invalid api-key")?.to_string()),
            None => None,
        };
        let datasets_dir: Option<PathBuf> = match value.1.get("datasets_dir") {
            Some(key) => Some(PathBuf::from(
                key.as_str().context("Invalid datasets_dir")?.to_string(),
            )),
            None => None,
        };

        Ok(Self {
            slug,
            datasets_dir,
            api_key,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str_all_fields() {
        let team_a: &'static str = "team-a:
        api_key: 1ed99664-726e-4400-bc5d-3132b22ce60c
        datasets_dir: /home/user/.v7/team-a
        ";

        let raw_team: serde_yaml::Value = serde_yaml::from_str(team_a).unwrap();
        let raw_team: Vec<(&Value, &Value)> = raw_team.as_mapping().unwrap().iter().collect();

        let team: Team = Team::try_from(*raw_team.first().unwrap()).unwrap();
        assert_eq!(team.slug(), "team-a".to_string());
        assert_eq!(
            team.api_key().as_ref().unwrap(),
            "1ed99664-726e-4400-bc5d-3132b22ce60c"
        );
        assert_eq!(
            team.datasets_dir().as_ref().unwrap(),
            &PathBuf::from("/home/user/.v7/team-a")
        );
    }

    #[test]
    fn test_from_str_slug_only() {
        let raw_team: serde_yaml::Value = serde_yaml::from_str("team-b:\n").unwrap();
        let raw_team: Vec<(&Value, &Value)> = raw_team.as_mapping().unwrap().iter().collect();

        let team: Team = Team::try_from(*raw_team.first().unwrap()).unwrap();
        assert_eq!(team.slug(), "team-b".to_string());
        assert_eq!(team.api_key().as_ref(), None);
        assert_eq!(team.datasets_dir().as_ref(), None);
    }
}
