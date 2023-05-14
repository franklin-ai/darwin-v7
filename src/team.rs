#[allow(unused_imports)]
use fake::{Dummy, Fake};

use crate::annotation::AnnotationClass;
use crate::expect_http_ok;
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use std::{fmt::Display, path::PathBuf};

use crate::client::V7Methods;

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct Team {
    pub slug: String,
    pub datasets_dir: Option<PathBuf>,
    pub api_key: Option<String>,
    pub team_id: Option<u32>,
}

#[derive(Debug, Default, Serialize, Deserialize, Dummy, PartialEq, Eq, Clone)]
pub struct TeamMember {
    pub id: u32,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub role: Option<String>,
    pub team_id: u32,
    pub user_id: u32,
}

impl Display for TeamMember {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{id-{}}}{} {} ({})",
            self.user_id,
            self.first_name.as_ref().unwrap_or(&String::new()),
            self.last_name.as_ref().unwrap_or(&String::new()),
            self.email.as_ref().unwrap_or(&String::new())
        )
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq, Eq)]
pub struct TypeCount {
    pub count: u32,
    pub id: Option<u32>,
    pub name: Option<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy)]
pub struct TeamAnnotationClasses {
    pub annotation_classes: Vec<AnnotationClass>,
    pub type_counts: Vec<TypeCount>,
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
            team_id: None,
        })
    }
}
#[async_trait]
pub trait TeamDescribeMethods<C>
where
    C: V7Methods,
{
    async fn list_memberships(client: &C) -> Result<Vec<TeamMember>>;
    async fn list_annotation_classes(&self, client: &C) -> Result<TeamAnnotationClasses>;
}

#[async_trait]
pub trait TeamDataMethods<C>
where
    C: V7Methods,
{
    async fn create_annotation_class(
        &self,
        client: &C,
        class: &AnnotationClass,
    ) -> Result<AnnotationClass>;
    // async fn delete_annotation_classes(
    //     &self,
    //     client: &C,
    //     classes: &[AnnotationClass],
    // ) -> Result<()>;
}

impl Team {
    pub fn new(
        slug: String,
        datasets_dir: Option<PathBuf>,
        api_key: Option<String>,
        team_id: Option<u32>,
    ) -> Self {
        Self {
            slug,
            datasets_dir,
            api_key,
            team_id,
        }
    }
}

#[async_trait]
impl<C> TeamDescribeMethods<C> for Team
where
    C: V7Methods + std::marker::Sync,
{
    // This uses the authentication token
    async fn list_memberships(client: &C) -> Result<Vec<TeamMember>> {
        let response = client.get("memberships").await?;

        expect_http_ok!(response, Vec<TeamMember>)
    }

    // Relies upon the team id / slug
    async fn list_annotation_classes(&self, client: &C) -> Result<TeamAnnotationClasses> {
        // TODO: add query params
        let endpoint = format!("teams/{}/annotation_classes", self.slug);
        let response = client.get(&endpoint).await?;

        expect_http_ok!(response, TeamAnnotationClasses)
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Dummy, PartialEq, Eq, Clone)]
struct DeleteClassesPayload {
    pub annotation_class_ids: Vec<u32>,
    pub annotations_to_delete_count: u32,
}

#[async_trait]
impl<C> TeamDataMethods<C> for Team
where
    C: V7Methods + std::marker::Sync,
{
    async fn create_annotation_class(
        &self,
        client: &C,
        class: &AnnotationClass,
    ) -> Result<AnnotationClass>
    where
        C: V7Methods,
    {
        let endpoint = format!("teams/{}/annotation_classes", self.slug);
        let response = client.post(&endpoint, class).await?;

        expect_http_ok!(response, AnnotationClass)
    }
    // async fn delete_annotation_classes(&self, client: &C, classes: &[AnnotationClass]) -> Result<()>
    // where
    //     C: V7Methods,
    // {
    //     let endpoint = format!(
    //         "teams/{}/delete_classes",
    //         self.team_id.context("Missing team id")?
    //     );

    //     let mut payload = DeleteClassesPayload::default();
    //     for class in classes.iter() {
    //         payload.annotation_class_ids.push(class.id.context(format!(
    //             "Class {} missing id",
    //             class.name.clone().unwrap_or(String::new())
    //         ))?);
    //     }

    //     let response = client.delete(&endpoint, &payload).await?;

    //     let status = response.status();
    //     if status != 204 {
    //         bail!("Unable to delete classes with status code {}", status);
    //     }

    //     Ok(())
    // }
}

#[derive(Debug, Clone, Serialize, Deserialize, Dummy, PartialEq, Eq)]
pub struct MetadataSkeleton {
    #[serde(rename = "_type")]
    pub skeleton_type: String,
}

pub mod helpers {
    use anyhow::Result;

    use crate::client::V7Methods;

    use super::{Team, TeamDescribeMethods, TeamMember};

    pub async fn find_team_members<C, F>(client: &C, func: F) -> Result<Vec<TeamMember>>
    where
        C: V7Methods + std::marker::Sync,
        F: Fn(&TeamMember) -> bool,
    {
        Ok(Team::list_memberships(client)
            .await?
            .iter()
            .filter(|x| func(x))
            .cloned()
            .collect::<Vec<TeamMember>>())
    }

    pub async fn find_team_members_by_email<C>(client: &C, email: &str) -> Result<Vec<TeamMember>>
    where
        C: V7Methods + std::marker::Sync,
    {
        find_team_members(client, |x| -> bool {
            x.email.as_ref().unwrap_or(&String::new()).contains(email)
        })
        .await
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
        assert_eq!(team.slug, "team-a".to_string());
        assert_eq!(
            team.api_key.as_ref().unwrap(),
            "1ed99664-726e-4400-bc5d-3132b22ce60c"
        );
        assert_eq!(
            team.datasets_dir.as_ref().unwrap(),
            &PathBuf::from("/home/user/.v7/team-a")
        );
    }

    #[test]
    fn test_from_str_slug_only() {
        let raw_team: serde_yaml::Value = serde_yaml::from_str("team-b:\n").unwrap();
        let raw_team: Vec<(&Value, &Value)> = raw_team.as_mapping().unwrap().iter().collect();

        let team: Team = Team::try_from(*raw_team.first().unwrap()).unwrap();
        assert_eq!(team.slug, "team-b".to_string());
        assert_eq!(team.api_key.as_ref(), None);
        assert_eq!(team.datasets_dir.as_ref(), None);
    }
}
