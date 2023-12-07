use crate::{
    annotation::{AnnotationClass, Keypoint, Tag},
    export::ImageAnnotation,
};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Struct representing the payload data wrapper of a V7 annotation suitable for importing back into a V7 dataset item
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AnnotationImportData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub polygon: Option<AnnotationImportPolygon>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<Tag>,
}

/// Struct representing the polygon payload data of a V7 annotation suitable for importing back into a V7 dataset item
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AnnotationImportPolygon {
    /// V7 provides no documentation on the format of these polygons on import.
    /// We assume though that these Keypoints are ordered in some way that represents a closed polygon.
    /// Typically, we import annotations as-is from V7 exports and retain the ordering as they were exported.
    /// This may change when we start merging polygons to import those merged polygons instead.
    pub path: Vec<Keypoint>,
}

/// Struct representing the context payload data of a V7 annotation suitable for importing back into a V7 dataset item
/// Identifies the slot names of the dataset item to import the annotation into
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AnnotationContext {
    /// List of slot IDs in the dataset item in V7 to attach the annotations to
    /// Although this field is called slot names, V7 actually expects slot IDs to be provided
    pub slot_names: Vec<String>,
}

/// Struct representing an annotation payload data of a V7 annotation suitable for importing back into a V7 dataset item
/// One instance of this struct corresponds to one annotation being imported into the dataset item
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AnnotationImportAnnotation {
    pub id: String,
    pub data: AnnotationImportData,
    pub annotation_class_id: u32,
    pub context_keys: AnnotationContext,
}

/// Struct representing a complete annotation import payload of V7 annotations into a single V7 dataset item
/// There will be one instance of this struct for each dataset item, where each struct contains many items for the annotations its importing
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AnnotationImport {
    pub annotations: Vec<AnnotationImportAnnotation>,
    pub overwrite: bool,
}

impl From<Vec<Keypoint>> for AnnotationImportPolygon {
    fn from(value: Vec<Keypoint>) -> Self {
        AnnotationImportPolygon { path: value }
    }
}

impl From<Vec<Keypoint>> for AnnotationImportData {
    fn from(value: Vec<Keypoint>) -> Self {
        AnnotationImportData {
            polygon: Some(AnnotationImportPolygon::from(value)),
            tag: None,
        }
    }
}

fn _find_annotation_class_id(
    eligible_annotation_classes: &[&AnnotationClass],
    class_name: &str,
) -> Result<u32> {
    eligible_annotation_classes
        .iter()
        .find(|ac| {
            if ac.name.is_some() {
                ac.name.clone().unwrap() == class_name
            } else {
                false
            }
        })
        .context("Unable to find matching annotation class ID from export JSON")?
        .id
        .context("Annotation Class has no ID")
}

impl AnnotationImportAnnotation {
    fn new_polygon_annotation(
        original_annotation: &ImageAnnotation,
        path: Vec<Keypoint>,
        eligible_annotation_classes: &[&AnnotationClass],
        slot_name: &str,
    ) -> Result<Self> {
        Ok(AnnotationImportAnnotation {
            id: uuid::Uuid::new_v4().to_string(),
            data: AnnotationImportData::from(path),
            annotation_class_id: _find_annotation_class_id(
                eligible_annotation_classes,
                &original_annotation.name,
            )?,
            context_keys: AnnotationContext {
                slot_names: vec![slot_name.to_string()],
            },
        })
    }

    fn new_tag_annotation(
        original_annotation: &ImageAnnotation,
        eligible_annotation_classes: &[&AnnotationClass],
        slot_name: &str,
    ) -> Result<Self> {
        Ok(AnnotationImportAnnotation {
            id: uuid::Uuid::new_v4().to_string(),
            data: AnnotationImportData {
                polygon: None,
                tag: original_annotation.tag.clone(),
            },
            annotation_class_id: _find_annotation_class_id(
                eligible_annotation_classes,
                &original_annotation.name,
            )?,
            context_keys: AnnotationContext {
                slot_names: vec![slot_name.to_string()],
            },
        })
    }
}
