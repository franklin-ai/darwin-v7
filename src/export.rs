//! This file contains structures and methods that define the Darwin Export Format
//! https://docs.v7labs.com/v1.0/reference/darwin-json

use crate::annotation::AnnotationType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Annotator {
    // Email of the Annotator or reviewer on Darwin
    pub email: String,
    // Full name (first name + last name) of the annotator
    // or reviewer
    pub full_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]

pub struct ImageExport {
    // Internal filename on Darwin
    pub filename: String,
    // Height of the image
    pub height: u32,
    // Original filename
    pub original_filename: String,
    // Path of file within Darwin
    pub path: String,
    // Sequence number is a monotonic increasing number
    // for each file uploaded int a Darwin Dataset.
    //seq: u64,
    // The URL of the image thumbnail
    pub thumbnail_url: String,
    //THe URL within V7 of the image
    pub url: String,
    // Width of the image
    pub width: u32,
    // The URL of the image on Darwin
    pub workview_url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImageAnnotation {
    // ID of the annotation
    pub id: Option<String>,
    // The annotation class name
    pub name: String,
    // An optional list of annotators of the image
    pub annotators: Option<Vec<Annotator>>,
    // An optional list of reviewers of the image
    pub reviewers: Option<Vec<Annotator>>,

    //// The actual data of the annotation
    #[serde(alias = "bounding_box", alias = "cuboid")]
    #[serde(alias = "skeleton", alias = "tag")]
    pub annotation_type_1: Option<AnnotationType>,

    #[serde(alias = "ellipse", alias = "line")]
    #[serde(alias = "keypoint", alias = "polygon")]
    pub annotation_type_2: Option<AnnotationType>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JsonExport {
    pub annotations: Vec<ImageAnnotation>,
    pub dataset: String,
    pub image: ImageExport,
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_v7_annotation_export_lines() -> Result<()> {
        let raw_json = r#"
                {
                  "annotators": [
                    {
                      "email": "abc.xyz@cheese.com",
                      "full_name": "ABC XYZ"
                    }
                  ],
                  "id": "fb81c35c-716a-413a-81e8-16ae9c054490",
                  "line": {
                      "path": [
                          {
                            "x": 103.92,
                            "y": 196.48
                          },
                          {
                            "x": 192.83,
                            "y": 123.58
                          }
                        ]
                  },
                  "name": "something"
                }
        "#;
        let _: ImageAnnotation = serde_json::from_str(raw_json)?;
        Ok(())
    }

    #[test]
    fn test_full_v7_export_file() -> Result<()> {
        let contents = r#"
        {
          "dataset": "Test Dataset",
          "image": {
            "filename": "xxxx-xxxx-xxxx-xxxx-xxxx.xxxx",
            "height": 88149,
            "original_filename": "xxxxx-xxxx-xxxx-xxxx-xxxx.xxxx",
            "path": "/",
            "thumbnail_url": "https://darwin.v7labs.com/api/images/999/thumbnail",
            "url": "https://darwin.v7labs.com/api/images/999/original",
            "width": 188688,
            "workview_url": "https://darwin.v7labs.com/workview?dataset=999&image=54"
          },
          "annotations": [
            {
              "bounding_box": {
                "h": 588.75,
                "w": 630.9500000000116,
                "x": 88527.01,
                "y": 11805.9
              },
              "id": "770e4a19-a350-4d5e-964e-783512a508f9",
              "name": "Cheese",
              "polygon": {
                "path": [
                  {
                    "x": 89094.67,
                    "y": 11924.8
                  }]
              }
            }
          ]
        }
        "#;
        let export: JsonExport = serde_json::from_str(contents).expect("Error parsing V7 Export");
        assert!(export.annotations[0].annotation_type_2.is_some());
        Ok(())
    }
}
