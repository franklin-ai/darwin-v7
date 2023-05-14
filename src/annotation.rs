#[allow(unused_imports)]
use fake::{Dummy, Fake};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strum_macros::{Display, EnumString};

use crate::client::V7Methods;
use crate::expect_http_ok;

#[derive(Debug, Clone, Serialize, Deserialize, Dummy, PartialEq, Eq)]
pub struct AnnotationClassMetadata {
    #[serde(rename = "_color")]
    pub color: String,
    pub polygon: Option<HashMap<String, String>>, // TODO find out what this type actually is
    pub auto_annotate: Option<HashMap<String, String>>, // TODO find out what this type actually is
    pub inference: Option<HashMap<String, String>>, // TODO find out what this type actually is
    pub measures: Option<HashMap<String, String>>, // TODO find out what this type actually is
}

#[derive(Debug, Clone, Serialize, Deserialize, Dummy, Default)]
pub struct BoundingBox {
    // Height of the bounding box
    pub h: f32,
    // Width of the bounding box
    pub w: f32,
    // Left-most coordinate of the bounding box
    pub x: f32,
    // Top-most coordinate of the bounding box
    pub y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Dummy, Default)]
pub struct Polygon {
    pub path: Vec<Keypoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Dummy, Default)]
pub struct Keypoint {
    // The horizontal coordinate of the keypoint
    pub x: f32,
    // The vertical coordinate of the key point
    pub y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Dummy, Default)]
pub struct Tag {}

#[derive(Debug, Clone, Serialize, Deserialize, Dummy, Default)]
pub struct Text {
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Dummy, Default)]
pub struct Line {
    pub path: Vec<Keypoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Dummy, EnumString, Display)]
#[serde(rename_all = "lowercase")]
#[serde(untagged)]
pub enum AnnotationType {
    Attributes,
    #[serde(rename = "auto_annotate")]
    AutoAnnotate,
    #[serde(rename = "bounding_box")]
    #[strum(serialize = "bounding_box")]
    BoundingBox(BoundingBox),
    Cuboid,
    #[serde(rename = "directional_vector")]
    DirectionalVector,
    #[strum(serialize = "ellipse")]
    Ellipse,
    Inference,
    #[serde(rename = "instance_id")]
    InstanceId,
    #[strum(serialize = "keypoint")]
    Keypoint(Keypoint),
    #[strum(serialize = "line")]
    Line(Line),
    Measures,
    #[strum(serialize = "polygon")]
    Polygon(Polygon),
    Skeleton,
    #[strum(serialize = "tag")]
    Tag(Tag),
    Text(Text),
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

impl From<AnnotationType> for u32 {
    fn from(value: AnnotationType) -> u32 {
        match value {
            AnnotationType::Attributes => 5,
            AnnotationType::AutoAnnotate => todo!(),
            AnnotationType::BoundingBox(_) => 2,
            AnnotationType::Cuboid => todo!(),
            AnnotationType::DirectionalVector => todo!(),
            AnnotationType::Ellipse => todo!(),
            AnnotationType::Inference => todo!(),
            AnnotationType::InstanceId => todo!(),
            AnnotationType::Keypoint(_) => todo!(),
            AnnotationType::Line(_) => 11,
            AnnotationType::Measures => todo!(),
            AnnotationType::Polygon(_) => 3,
            AnnotationType::Skeleton => 12,
            AnnotationType::Tag(_) => 1,
            AnnotationType::Text(_) => 6,
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq, Eq)]
pub struct AnnotationDataset {
    pub id: u32,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy)]
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

    pub async fn delete<C>(&self, client: &C) -> Result<()>
    where
        C: V7Methods,
    {
        let endpoint = format!(
            "annotation_classes/{}",
            self.id.context("Annotation class is missing an id")?
        );

        let response = client.delete::<AnnotationClass>(&endpoint, None).await?;

        if response.status() != 204 {
            bail!(format!(
                "Invalid status code {} {}",
                response.status(),
                response.text().await?
            ));
        }

        Ok(())
    }
}
