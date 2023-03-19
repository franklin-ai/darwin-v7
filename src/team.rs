use crate::expect_http_ok;
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use fake::{Dummy, Fake};
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use std::collections::HashMap;
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
    async fn delete_annotation_classes(
        &self,
        client: &C,
        classes: &[AnnotationClass],
    ) -> Result<()>;
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
    async fn delete_annotation_classes(&self, client: &C, classes: &[AnnotationClass]) -> Result<()>
    where
        C: V7Methods,
    {
        let endpoint = format!(
            "teams/{}/delete_classes",
            self.team_id.context("Missing team id")?
        );

        let mut payload = DeleteClassesPayload::default();
        for class in classes.iter() {
            payload.annotation_class_ids.push(class.id.context(format!(
                "Class {} missing id",
                class.name.clone().unwrap_or(String::new())
            ))?);
        }

        let response = client.delete(&endpoint, &payload).await?;

        let status = response.status();
        if status != 204 {
            bail!("Unable to delete classes with status code {}", status);
        }

        Ok(())
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
pub enum AnnotationType {
    Attributes,
    #[serde(rename = "auto_annotate")]
    AutoAnnotate,
    #[serde(rename = "bounding_box")]
    BoundingBox,
    Cuboid,
    #[serde(rename = "directional_vector")]
    DirectionalVector,
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

// Various ids for annotation types and sub types
// Tag: 1
// Tag-Attributes: [5 1]
// Tag-Text: [6, 1]
// Tag-Attributes-Text: [5, 6, 1]
// Polygon: 3
// Polygon-Attributes: [5, 3]
// Polygon-Text: [6, 3]
// Polygon-DirectionalVector: [6 4 3]
// Polygon-InstanceId: [9 3]
// bbox: 2
// bbox-attributes: [2 5]
// bbox-tag: [2, 6]
// skeleton: 12
// skeleton-text: [12 6]
// line: 11
// line-text-instanceid: [6 9 11]
// keypoint: 7
// ellipse: 60
// cuboid: 8

impl Into<u32> for AnnotationType {
    fn into(self) -> u32 {
        match self {
            AnnotationType::Attributes => 5,
            AnnotationType::AutoAnnotate => todo!(),
            AnnotationType::BoundingBox => 2,
            AnnotationType::Cuboid => todo!(),
            AnnotationType::DirectionalVector => todo!(),
            AnnotationType::Ellipse => todo!(),
            AnnotationType::Inference => todo!(),
            AnnotationType::InstanceId => todo!(),
            AnnotationType::Keypoint => todo!(),
            AnnotationType::Line => 11,
            AnnotationType::Measures => todo!(),
            AnnotationType::Polygon => 3,
            AnnotationType::Skeleton => 12,
            AnnotationType::Tag => 1,
            AnnotationType::Text => 6,
        }
    }
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
    pub annotation_types: Vec<AnnotationType>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotation_type_ids: Option<Vec<u32>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub dataset_id: Option<u32>,

    // #[serde(skip_serializing_if = "Vec::is_empty")]
    pub datasets: Vec<AnnotationDataset>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_id: Option<u32>,

    // #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    // #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Vec<String>, // TODO: find out what this type is

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
            team_id: None,
        })
    }
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
            .filter(|x| func(*x))
            .map(|x| x.clone())
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
