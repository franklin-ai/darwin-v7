//! This file contains structures and methods that define the Darwin Export Format
//! https://docs.v7labs.com/v1.0/reference/darwin-json

use crate::annotation::AnnotationType;
use crate::item::DatasetItemTypes;
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

    #[serde(alias = "keypoint", alias = "polygon", alias = "complex_polygon")]
    pub annotation_type_2: Option<AnnotationType>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JsonExport {
    pub annotations: Vec<ImageAnnotation>,
    pub dataset: String,
    pub image: ImageExport,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Item {
    name: String,
    path: String,
    source_info: SourceInfo,
    slots: Vec<Slot>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct SourceInfo {
    dataset: Dataset,
    item_id: String,
    team: Team,
    workview_url: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Dataset {
    name: String,
    slug: String,
    dataset_management_url: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Team {
    name: String,
    slug: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Slot {
    #[serde(rename = "type")]
    slot_type: DatasetItemTypes,
    slot_name: String,
    width: u32,
    height: u32,
    thumbnail_url: String,
    source_files: Vec<SourceFile>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
struct SourceFile {
    file_name: String,
    storage_key: String,
    url: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct JsonExportV2 {
    pub version: String,
    pub schema_ref: String,
    pub item: Item,
    pub annotations: Vec<ImageAnnotation>,
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
                  "bounding_box": {
                    "h": 588.75,
                    "w": 630.9500000000116,
                    "x": 88527.01,
                    "y": 11805.9
                  },
                  "complex_polygon": {
                    "path": [
                    [
                      {
                        "x": 89094.67,
                        "y": 11924.8
                      }], [
                      {
                        "x": 89094.67,
                        "y": 11924.8
                      }]]
                  },
                  "name": "something"
                }
        "#;
        let annotation: ImageAnnotation = serde_json::from_str(raw_json)?;
        match annotation.annotation_type_2 {
            Some(AnnotationType::ComplexPolygon(_)) => {}
            _ => {
                assert!(false);
            }
        };
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

    #[test]
    fn test_full_v7_export_v2_file() -> Result<()> {
        let contents = r#"
        {
          "version": "2.0",
          "schema_ref": "https://darwin-public.s3.eu-west-1.amazonaws.com/darwin_json/2.0/schema.json",
          "item": {
            "name": "bf007a29-6559-d0cc-c549-45c7c66d4c70.e47f119",
            "path": "/",
            "source_info": {
              "dataset": {
                "name": "V7 Api V2 Testing - 01-01-1990 - Fake Pathologist 1 - fake.pathologist_1@franklin.ai",
                "slug": "v7-api-v2-testing-01-01-1990-fake-pathologist-1-fake-pathologist_1-franklin-ai",
                "dataset_management_url": "https://darwin.v7labs.com/datasets/669290/dataset-management"
              },
              "item_id": "0189b92f-e00c-fea9-476c-0cb6e961362b",
              "team": {
                "name": "V7 Api v2 Testing",
                "slug": "v7-api-v2-testing"
              },
              "workview_url": "https://darwin.v7labs.com/workview?dataset=669290&item=0189b92f-e00c-fea9-476c-0cb6e961362b"
            },
            "slots": [
              {
                "type": "image",
                "slot_name": "bf007a29-6559-d0cc-c549-45c7c66d4c70.e47f119",
                "width": 156945,
                "height": 66467,
                "thumbnail_url": "https://darwin.v7labs.com/api/v2/teams/v7-api-v2-testing/files/bc8bd76b-6280-4136-a4ed-904b863e3133/thumbnail",
                "source_files": [
                  {
                    "file_name": "bf007a29-6559-d0cc-c549-45c7c66d4c70.e47f119",
                    "storage_key": "images/20220704_AU1_List-6/Leica_Scans/AU1/7FF79C60EC2FD73FF16F73C1420591BDD2169B5057C5F9766E1F7589DB73A88C/6BC8E4510F1E895D2A8B8C807F46E810A19D53F52278918C10A6B2AC7AF573A6/bf007a29-6559-d0cc-c549-45c7c66d4c70.e47f119.fra",
                    "url": "https://darwin.v7labs.com/api/v2/teams/v7-api-v2-testing/uploads/75277109-8c0d-4c4d-9969-1939f96f25ba"
                  }
                ]
              }
            ]
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
              },

              "reviewers": [
                {
                  "email": "fake.pathologist@franklin.ai",
                  "full_name": "Fake Pathologist"
                }
              ],
              "slot_names": [
                "bf007a29-6559-d0cc-c549-45c7c66d4c70.e47f119"
              ],
              "updated_at": "2023-08-03T03:04:37"
            }
          ]
        }
        "#;
        let export: JsonExportV2 = serde_json::from_str(contents).expect("Error parsing V7 Export");
        assert_eq!(export.version, "2.0");
        assert!(export.annotations[0].annotation_type_2.is_some());
        Ok(())
    }
}
