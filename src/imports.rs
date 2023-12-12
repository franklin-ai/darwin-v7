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

impl AnnotationImportAnnotation {
    /// Creates a new polygon annotation.
    ///
    /// This function generates an `AnnotationImportAnnotation` instance representing a polygon annotation.
    /// It assigns a unique ID, sets the annotation data with the provided `path` representing the polygon,
    /// and links it to an appropriate annotation class based on the `original_annotation` and the provided
    /// `eligible_annotation_classes`. The `slot_name` is used to specify where in the dataset item this annotation
    /// should be attached.
    ///
    /// # Arguments
    ///
    /// * `original_annotation` - A reference to an `ImageAnnotation` from which the name of the annotation class is derived.
    /// * `path` - A vector of `Keypoint` objects defining the vertices of the polygon.
    /// * `eligible_annotation_classes` - A slice of references to `AnnotationClass` objects.
    ///    The function searches these to find a matching class ID for the `original_annotation`.
    /// * `slot_name` - The name of the slot in the dataset item where this annotation will be attached.
    ///
    /// # Returns
    ///
    /// Returns `Result<AnnotationImportAnnotation, Error>` where `Ok` contains the newly created `AnnotationImportAnnotation`
    /// if the annotation class corresponding to `original_annotation` is found in `eligible_annotation_classes`;
    /// otherwise, an error is returned.
    ///
    /// # Errors
    ///
    /// Returns an error if no matching annotation class ID is found in `eligible_annotation_classes` for the `original_annotation`.
    pub fn new_polygon_annotation(
        original_annotation: &ImageAnnotation,
        path: Vec<Keypoint>,
        eligible_annotation_classes: &[&AnnotationClass],
        slot_name: &str,
    ) -> Result<Self> {
        Ok(AnnotationImportAnnotation {
            id: uuid::Uuid::new_v4().to_string(),
            data: AnnotationImportData::from(path),
            annotation_class_id: Self::find_annotation_class_id(
                eligible_annotation_classes,
                &original_annotation.name,
            )?,
            context_keys: AnnotationContext {
                slot_names: vec![slot_name.to_string()],
            },
        })
    }

    /// Creates a new tag annotation.
    ///
    /// This function generates an `AnnotationImportAnnotation` instance for a tag annotation.
    /// It assigns a unique ID, sets the annotation data based on the `original_annotation`,
    /// and identifies the correct annotation class from `eligible_annotation_classes`.
    /// The `slot_name` is used to specify the dataset item slot for the annotation.
    ///
    /// # Arguments
    ///
    /// * `original_annotation` - A reference to an `ImageAnnotation` used to derive the tag data
    ///   and annotation class name.
    /// * `eligible_annotation_classes` - A slice of references to `AnnotationClass` objects.
    ///   The function searches these to find a matching class ID for the `original_annotation`.
    /// * `slot_name` - The name of the slot in the dataset item where this annotation will be attached.
    ///
    /// # Returns
    ///
    /// Returns `Result<AnnotationImportAnnotation, Error>`. On success, it contains the newly
    /// created `AnnotationImportAnnotation`. If a matching annotation class is not found in
    /// `eligible_annotation_classes`, it returns an error.
    ///
    /// # Errors
    ///
    /// Returns an error if no matching annotation class ID is found in `eligible_annotation_classes`
    /// for the `original_annotation`.
    pub fn new_tag_annotation(
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
            annotation_class_id: Self::find_annotation_class_id(
                eligible_annotation_classes,
                &original_annotation.name,
            )?,
            context_keys: AnnotationContext {
                slot_names: vec![slot_name.to_string()],
            },
        })
    }

    /// Finds the ID of an annotation class.
    ///
    /// This function searches through a slice of `AnnotationClass` references to find a class
    /// whose name matches the provided `class_name`. It is primarily used to retrieve the unique
    /// ID associated with an annotation class based on its name.
    ///
    /// # Arguments
    ///
    /// * `eligible_annotation_classes` - A slice of references to `AnnotationClass` objects in which
    ///   the function searches for a matching class name.
    /// * `class_name` - The name of the annotation class for which the ID is being searched.
    ///
    /// # Returns
    ///
    /// Returns `Result<u32, Error>`. On success, it contains the ID of the matching annotation class.
    /// If no class with the given name is found, or if the found class lacks an ID, it returns an error.
    ///
    /// # Errors
    ///
    /// Returns an error in the following cases:
    /// - No annotation class with the given name is found in `eligible_annotation_classes`.
    /// - The annotation class found does not have an ID.
    fn find_annotation_class_id(
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::annotation::{AnnotationClass, Keypoint};
    use crate::imports::ImageAnnotation;
    use anyhow::Result;

    // A helper function to create a sample ImageAnnotation for testing
    fn create_sample_image_annotation(tag: Option<Tag>) -> ImageAnnotation {
        ImageAnnotation {
            name: "Sample Class".to_string(),
            tag,
            ..ImageAnnotation::default()
        }
    }

    // A helper function to create a sample AnnotationClass for testing
    fn create_sample_annotation_class(name: &str, id: u32) -> AnnotationClass {
        AnnotationClass {
            name: Some(name.to_string()),
            id: Some(id),
            ..AnnotationClass::default()
        }
    }

    #[test]
    fn test_new_polygon_annotation_success() -> Result<()> {
        let original_annotation = create_sample_image_annotation(None);
        let path = vec![Keypoint { x: 10.0, y: 10.0 }, Keypoint { x: 20.0, y: 20.0 }];
        let eligible_annotation_classes = &[&create_sample_annotation_class("Sample Class", 1)];

        let result = AnnotationImportAnnotation::new_polygon_annotation(
            &original_annotation,
            path,
            eligible_annotation_classes,
            "sample_slot",
        )?;

        assert_eq!(result.id.len(), 36); // UUID length
        assert!(result.data.polygon.is_some());
        assert_eq!(result.annotation_class_id, 1);
        assert_eq!(
            result.context_keys.slot_names,
            vec!["sample_slot".to_string()]
        );

        Ok(())
    }

    #[test]
    fn test_new_polygon_annotation_with_invalid_class() {
        let original_annotation = create_sample_image_annotation(None);
        let path = vec![Keypoint { x: 10.0, y: 10.0 }, Keypoint { x: 20.0, y: 20.0 }];
        let eligible_annotation_classes = &[]; // No eligible classes

        let result = AnnotationImportAnnotation::new_polygon_annotation(
            &original_annotation,
            path,
            eligible_annotation_classes,
            "sample_slot",
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_new_tag_annotation_success() -> Result<()> {
        let original_annotation = create_sample_image_annotation(Some(Tag {}));
        let eligible_annotation_classes = &[&create_sample_annotation_class("Sample Class", 1)];

        let result = AnnotationImportAnnotation::new_tag_annotation(
            &original_annotation,
            eligible_annotation_classes,
            "sample_slot",
        )?;

        assert_eq!(result.id.len(), 36); // UUID length
        assert!(result.data.tag.is_some());
        assert_eq!(result.annotation_class_id, 1);
        assert_eq!(
            result.context_keys.slot_names,
            vec!["sample_slot".to_string()]
        );

        Ok(())
    }

    #[test]
    fn test_new_tag_annotation_with_invalid_class() {
        let original_annotation = create_sample_image_annotation(Some(Tag {}));
        let eligible_annotation_classes = &[]; // No eligible classes

        let result = AnnotationImportAnnotation::new_tag_annotation(
            &original_annotation,
            eligible_annotation_classes,
            "sample_slot",
        );

        assert!(result.is_err());
    }
}
