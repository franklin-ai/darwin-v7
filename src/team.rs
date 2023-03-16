use crate::expect_http_ok;
use anyhow::{bail, Context, Result};
use fake::{Dummy, Fake};
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use std::collections::HashMap;
use std::{fmt::Display, path::PathBuf};

use crate::client::V7Methods;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Team {
    slug: String,
    datasets_dir: Option<PathBuf>,
    api_key: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq, Clone)]
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

impl Team {
    pub fn new(slug: String, datasets_dir: Option<PathBuf>, api_key: Option<String>) -> Self {
        Self {
            slug,
            datasets_dir,
            api_key,
        }
    }
    pub fn slug(&self) -> &str {
        &self.slug
    }

    pub fn datasets_dir(&self) -> &Option<PathBuf> {
        &self.datasets_dir
    }

    pub fn api_key(&self) -> &Option<String> {
        &self.api_key
    }

    pub async fn list_memberships<C>(client: &C) -> Result<Vec<TeamMember>>
    where
        C: V7Methods,
    {
        let response = client.get("memberships").await?;

        expect_http_ok!(response, Vec<TeamMember>)
    }

    pub async fn list_annotation_classes<C>(&self, client: &C) -> Result<TeamAnnotationClasses>
    where
        C: V7Methods,
    {
        // TODO: add query params
        let endpoint = format!("teams/{}/annotation_classes", self.slug);
        let response = client.get(&endpoint).await?;

        expect_http_ok!(response, TeamAnnotationClasses)
    }

    pub async fn create_annotation_class<C>(
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
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq)]
pub struct MetadataSkeleton {
    #[serde(rename = "_type")]
    pub skeleton_type: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq)]
pub struct AnnotationClassMetadata {
    #[serde(rename = "_color")]
    pub color: String,
    pub polygon: Option<HashMap<String, String>>, // TODO find out what this type actually is
    pub auto_annotate: Option<HashMap<String, String>>, // TODO find out what this type actually is
    pub inference: Option<HashMap<String, String>>, // TODO find out what this type actually is
    pub measures: Option<HashMap<String, String>>, // TODO find out what this type actually is
}

#[derive(Debug, Clone, Serialize, Deserialize, Dummy, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AnnotationTypes {
    Attributes,
    #[serde(rename = "auto_annotate")]
    AutoAnnotate,
    #[serde(rename = "bounding_box")]
    BoundingBox,
    Cuboid,
    Ellipse,
    Inference,
    #[serde(rename = "instance_id")]
    InstanceId,
    Keypoint,
    Line,
    Measures,
    Polygon,
    Skeleton,
    Tag,
    Text,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq)]
pub struct AnnotationDataset {
    pub id: u32,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq)]
pub struct AnnotationClass {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotation_class_image_url: Option<String>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub annotation_types: Vec<AnnotationTypes>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub dataset_id: Option<u32>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub datasets: Vec<AnnotationDataset>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_id: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<String>>, // TODO: find out what this type is

    #[serde(skip_serializing_if = "Option::is_none")]
    pub inserted_at: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<AnnotationClassMetadata>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

impl AnnotationClass {
    pub async fn update<C>(&self, client: &C) -> Result<AnnotationClass>
    where
        C: V7Methods,
    {
        let endpoint = format!(
            "annotation_classes/{}",
            self.id.context("Annotation class is missing an id")?
        );
        let response = client.put(&endpoint, Some(&self)).await?;

        expect_http_ok!(response, AnnotationClass)
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq)]
pub struct TypeCount {
    pub count: u32,
    pub id: Option<u32>,
    pub name: Option<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq)]
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
        })
    }
}

pub mod helpers {
    use anyhow::Result;

    use crate::client::V7Methods;

    use super::{Team, TeamMember};

    pub async fn find_team_members<C, F>(client: &C, func: F) -> Result<Vec<TeamMember>>
    where
        C: V7Methods,
        F: Fn(&TeamMember) -> bool,
    {
        Ok(Team::list_memberships(client)
            .await?
            .iter()
            .filter(|x| func(*x))
            .map(|x| x.clone())
            .collect::<Vec<TeamMember>>())
    }

    pub async fn find_team_members_by_email<C>(client: &C, email: &str) -> Result<Vec<TeamMember>>
    where
        C: V7Methods,
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
