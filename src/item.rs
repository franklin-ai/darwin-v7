use crate::workflow::StageType;
use anyhow::{bail, Result};
use fake::{Dummy, Fake, Faker};
use serde::ser::SerializeMap;
use serde::{de::MapAccess, de::Visitor, Deserialize, Deserializer, Serialize, Serializer};
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::fmt::{self, Display};

// Most fields in the following structures are listed as optional
// as depending upon the context the V7 endpoint may return a null
// value.  There is a high degree of variability as to when and why
// a null may be provided in the JSON payload.

#[derive(Debug, Clone, Serialize, Deserialize, Dummy)]
pub struct ImageLevel {
    pub format: String,
    pub pixel_ratio: u16,
    pub tile_height: u32,
    pub tile_width: u32,
    pub x_tiles: f32,
    pub y_tiles: f32,
}

impl PartialEq<Self> for ImageLevel {
    fn eq(&self, other: &Self) -> bool {
        self.format == other.format
            && self.pixel_ratio == other.pixel_ratio
            && self.tile_width == other.tile_width
            && self.tile_height == other.tile_height
            && (self.x_tiles.is_nan() == other.x_tiles.is_nan() || self.x_tiles.eq(&other.x_tiles))
            && (self.y_tiles.is_nan() == other.y_tiles.is_nan() || self.y_tiles.eq(&other.y_tiles))
    }
}

impl Eq for ImageLevel {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Levels {
    pub image_levels: HashMap<u32, ImageLevel>,
    pub base_key: Option<String>,
}

// JSON levels have a mix of ImageLevel information as well
// as the base_key.  A custom serializer / deserializer
// is required due to the mix of data types in the json structures
// e.g.
// {
//     "0": {
//         "format": "png",
//         "pixel_ratio": 1,
//         "tile_height": 2048,
//         "tile_width": 2048,
//         "x_tiles": 82,
//         "y_tiles": 22
//     },
//     "base_key": "some-base-key.jpg"
// }"

impl Serialize for Levels {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let map_length = self.image_levels.len() + 1; // Levels + base_key
        let mut seq = serializer.serialize_map(Some(map_length))?;

        for (key, val) in self.image_levels.iter() {
            seq.serialize_entry(key, val)?;
        }

        seq.serialize_entry("base_key", &self.base_key)?;
        seq.end()
    }
}

impl<'de> Deserialize<'de> for Levels {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(LevelVisitor)
    }
}

struct LevelVisitor;

impl<'de> Visitor<'de> for LevelVisitor {
    type Value = Levels;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a map with keys '0'..'N' and 'base_key'")
    }

    fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut image_levels: HashMap<u32, ImageLevel> = HashMap::new();
        let mut base_key = String::new();

        while let Some(k) = map.next_key::<&str>()? {
            if k == "base_key" {
                base_key = map.next_value::<String>()?;
            } else {
                let level_key = match k.parse::<u32>() {
                    Ok(val) => val,
                    Err(_) => return Err(serde::de::Error::custom(format!("Invalid key: {k}"))),
                };

                let level: ImageLevel = map.next_value()?;

                image_levels.insert(level_key, level);
            }
        }

        Ok(Levels {
            image_levels,
            base_key: Some(base_key),
        })
    }
}

impl Dummy<fake::Faker> for Levels {
    fn dummy_with_rng<R: fake::rand::RngCore + ?Sized>(_: &fake::Faker, rng: &mut R) -> Self {
        let max_levels: u32 = (2..5).fake_with_rng(rng);
        let base_key: Option<String> = Faker.fake_with_rng(rng);

        let mut image_levels = HashMap::new();
        for lvl in 1..max_levels {
            let img_level: ImageLevel = Faker.fake_with_rng(rng);
            let lvl = lvl - 1;
            image_levels.insert(lvl, img_level);
        }

        Self {
            image_levels,
            base_key,
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq, Eq)]
pub struct Image {
    pub external: Option<bool>,
    pub format: Option<String>,
    pub height: Option<u32>,
    pub width: Option<u32>,
    pub id: Option<u32>,
    pub key: Option<String>,
    pub levels: Option<Levels>,
    pub original_filename: Option<String>,
    pub thumbnail_url: Option<String>,
    pub uploaded: Option<bool>,
    pub url: Option<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq, Eq)]
pub struct DatasetImage {
    pub dataset_id: Option<u32>,
    pub dataset_video_id: Option<u32>,
    pub id: Option<u32>,
    pub image: Option<Image>,
    pub seq: Option<u32>,
    pub set: Option<u32>,
}

// TODO: Define this struct
#[derive(Debug, Clone, Serialize, Deserialize, Dummy, PartialEq, Eq)]
pub struct DatasetVideo {}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DatasetItemTypes {
    #[default]
    Image,
    Video,
    Pdf,
    Dicom,
    TiledImage,
}

impl Display for DatasetItemTypes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DatasetItemTypes::Image => write!(f, "Image"),
            DatasetItemTypes::Video => write!(f, "Video"),
            DatasetItemTypes::Pdf => write!(f, "PDF"),
            DatasetItemTypes::Dicom => write!(f, "DICOM"),
            DatasetItemTypes::TiledImage => write!(f, "tiled_image"),
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DatasetItemStatus {
    Annotate,
    Archived,
    Complete,
    Error,
    #[default]
    New,
    Processing,
    Review,
    Uploading,
}

impl Display for DatasetItemStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DatasetItemStatus::Annotate => write!(f, "Annotate"),
            DatasetItemStatus::Archived => write!(f, "Archived"),
            DatasetItemStatus::Complete => write!(f, "Complete"),
            DatasetItemStatus::Error => write!(f, "Error"),
            DatasetItemStatus::New => write!(f, "New"),
            DatasetItemStatus::Processing => write!(f, "Processing"),
            DatasetItemStatus::Review => write!(f, "Review"),
            DatasetItemStatus::Uploading => write!(f, "Uploading"),
        }
    }
}

impl TryFrom<&str> for DatasetItemStatus {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, <DatasetItemStatus as TryFrom<&str>>::Error> {
        Ok(match value.to_lowercase().as_str() {
            "annotate" => Self::Annotate,
            "archived" => Self::Archived,
            "complete" => Self::Complete,
            "error" => Self::Error,
            "new" => Self::New,
            "processing" => Self::Processing,
            "review" => Self::Review,
            "uploading" => Self::Uploading,
            _ => bail!("Cannot convert DatasetItemStatus from {value}"),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Dummy, PartialEq, Eq)]
pub struct DataPayloadLevel {
    pub levels: HashMap<usize, ImageLevel>,
    pub base_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Dummy, PartialEq, Eq)]
pub struct AddDataPayload {
    #[serde(rename = "type")]
    pub item_type: DatasetItemTypes,
    pub filename: String,
    pub thumbnail_key: String,
    pub path: String,
    pub key: String,
    pub width: u32,
    pub height: u32,
    pub metadata: DataPayloadLevel,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq, Eq)]
pub struct NewSimpleItem {
    pub as_frames: bool,
    pub extract_views: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fps: Option<String>, // is either a positive integer number or the string `native`
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, String>,
    pub name: String,
    pub path: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>, // TODO: https://docs.v7labs.com/reference/imports-upload tags in the json object can either be Vec<String> or HashMap<String, String>
    #[serde(rename = "type")]
    pub typ: DatasetItemTypes,
}

#[derive(Debug, Clone, Serialize, Deserialize, Dummy, PartialEq, Eq)]
pub struct RegisterNewItemOptions {
    pub force_tiling: bool,
    pub ignore_dicom_layout: bool,
}

impl Default for RegisterNewItemOptions {
    fn default() -> Self {
        Self {
            force_tiling: false,
            ignore_dicom_layout: true,
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq, Eq)]
pub struct RegisterNewSimpleItemRequest {
    pub dataset_slug: String,
    pub items: Vec<NewSimpleItem>,
    pub options: RegisterNewItemOptions,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq, Eq)]
pub struct ImageSection {
    pub height: u32,
    pub width: u32,
    pub size_bytes: u32,
    pub section_index: usize,
    pub storage_hq_key: String,
    #[serde(rename = "type")]
    pub image_section_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Dummy, PartialEq, Eq)]
pub struct Slot {
    pub sections: Vec<ImageSection>,
    pub file_name: String,
    pub size_bytes: u32,
    pub slot_name: String,
    pub storage_key: String,
    pub storage_thumbnail_key: String,
    #[serde(rename = "type")]
    pub slot_type: DatasetItemTypes,
    pub metadata: DataPayloadLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize, Dummy, PartialEq, Eq)]
pub struct ExistingSimpleItem {
    pub name: String,
    pub path: String,
    pub slots: Vec<Slot>,
}

impl Display for DatasetItemV2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{id-{:?}}}:{:?}/{:?}[{:?}]",
            self.id, self.name, self.status, self.slot_types
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Dummy, PartialEq, Eq)]
pub struct ItemSlotMetadata {
    pub levels: Option<Levels>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Dummy)]
pub struct ItemSlot {
    pub file_name: Option<String>,
    pub fps: Option<f32>,
    pub id: Option<String>,
    pub is_external: Option<bool>,
    pub metadata: Option<ItemSlotMetadata>,
    pub size_bytes: Option<u64>,
    pub slot_name: Option<String>,
    pub streamable: Option<bool>,
    pub total_sections: Option<u32>,
    #[serde(rename = "type")]
    pub item_slot_type: Option<DatasetItemTypes>,
    pub upload_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub legacy_item_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Dummy)]
pub struct DatasetItemLayout {
    pub slots: Vec<Option<String>>,
    #[serde(rename = "type")]
    pub layout_type: Option<String>,
    pub version: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Dummy)]
pub struct ProcessingError {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processing_error_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http_status_code: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_error: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Dummy)]
pub struct DatasetItemUploads {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upload_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processing_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upload_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processing_error: Option<ProcessingError>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub as_frames: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Dummy)]
pub struct DatasetItemV2 {
    pub archived: Option<bool>,
    pub cursor: Option<String>,
    pub dataset_id: Option<u32>,
    pub id: Option<String>,
    pub inserted_at: Option<String>,
    pub layout: Option<DatasetItemLayout>,
    pub name: Option<String>,
    pub path: Option<String>,
    pub priority: Option<u32>,
    pub processing_status: Option<DatasetItemStatus>,
    pub slot_types: Vec<Option<DatasetItemTypes>>,
    pub slots: Vec<Option<ItemSlot>>,
    pub status: Option<DatasetItemStatus>,
    pub tags: Vec<Option<String>>,
    pub updated_at: Option<String>,
    pub uploads: Vec<Option<DatasetItemUploads>>,
    pub workflow_status: Option<StageType>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Dummy)]
pub struct ItemPage {
    pub count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,
    pub previous: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Dummy)]
pub struct Item {
    pub items: Vec<Option<DatasetItemV2>>,
    pub page: ItemPage,
}

#[cfg(test)]
mod test_serde {
    use super::*;

    #[test]
    fn test_levels_dummy() {
        let level: Levels = Faker.fake();
        assert!(level.image_levels.keys().len() >= 1);
    }

    #[test]
    fn test_image_level_ser_deser() {
        // Test serialization
        let contents = r#"
        {
            "0": {
                "format": "png",
                "pixel_ratio": 1,
                "tile_height": 2048,
                "tile_width": 2048,
                "x_tiles": 82.0,
                "y_tiles": 22.0
            },
            "base_key": "some-base-key.jpg"
        }"#;

        let image_level: Levels = serde_json::from_str(contents).unwrap();

        assert_eq!(image_level.base_key, Some("some-base-key.jpg".to_string()));
        assert_eq!(
            image_level.image_levels.get(&0).unwrap().format,
            "png".to_string()
        );

        // Test deserialization
        let image_level_deser = serde_json::to_string(&image_level).unwrap();

        assert_eq!(&image_level_deser, &contents.replace(['\n', ' '], ""));
    }

    #[test]
    fn test_image_level_ser_deser_error() {
        // Test serialization
        let contents = r#"
        {
            "an-image": {
                "format": "png",
                "pixel_ratio": 1,
                "tile_height": 2048,
                "tile_width": 2048,
                "x_tiles": 82.0,
                "y_tiles": 22.0
            },
            "base_key": "some-base-key.jpg"
        }"#;
        serde_json::from_str::<Levels>(contents).expect_err("Invalid key: an-image");

        serde_json::from_str::<Levels>(r#"{"key": "value"}"#)
            .expect_err("a map with keys '0'..'N' and 'base_key'");
    }

    #[test]
    fn test_dataset_item_v2() {
        let contents = r#"
    {
          "archived": false,
          "cursor": "018951a8-466f-d61d-9c3b-45ba10201cc3",
          "dataset_id": 657106,
          "id": "018951a8-466f-d61d-9c3b-45ba10201cc3",
          "inserted_at": "2023-07-13T23:48:49Z",
          "layout": {
            "slots": [
              "1cb65790-1e55-cb67-501d-a48b8ca05535.e47f119"
            ],
            "type": "simple",
            "version": 1
          },
          "name": "1cb65790-1e55-cb67-501d-a48b8ca05535.e47f119",
          "path": "/",
          "priority": 0,
          "processing_status": "complete",
          "slot_types": [
            "image",
            "image"
          ],
          "slots": [
            {
              "file_name": "1cb65790-1e55-cb67-501d-a48b8ca05535.e47f119",
              "fps": 1,
              "id": "8ada6682-48fd-464f-9f9d-28b6f4f81591",
              "is_external": true,
              "metadata": {
                "base_key": "images/List-5/XYZ_Scans/AU1/ABCD/XYZ/123-344-456-678-123.454.fra/",
                "height": 88539,
                "levels": {
                  "0": {
                    "format": "png",
                    "pixel_ratio": 1,
                    "tile_height": 2048,
                    "tile_width": 2048,
                    "x_tiles": 79,
                    "y_tiles": 44
                  },
                  "1": {
                    "format": "png",
                    "pixel_ratio": 2,
                    "tile_height": 2048,
                    "tile_width": 2048,
                    "x_tiles": 40,
                    "y_tiles": 22
                  },
                  "2": {
                    "format": "png",
                    "pixel_ratio": 4,
                    "tile_height": 2048,
                    "tile_width": 2048,
                    "x_tiles": 20,
                    "y_tiles": 11
                  },
                  "3": {
                    "format": "png",
                    "pixel_ratio": 8,
                    "tile_height": 2048,
                    "tile_width": 2048,
                    "x_tiles": 10,
                    "y_tiles": 6
                  },
                  "4": {
                    "format": "png",
                    "pixel_ratio": 16,
                    "tile_height": 2048,
                    "tile_width": 2048,
                    "x_tiles": 5,
                    "y_tiles": 3
                  },
                  "5": {
                    "format": "png",
                    "pixel_ratio": 32,
                    "tile_height": 2048,
                    "tile_width": 2048,
                    "x_tiles": 3,
                    "y_tiles": 2
                  },
                  "6": {
                    "format": "png",
                    "pixel_ratio": 64,
                    "tile_height": 2048,
                    "tile_width": 2048,
                    "x_tiles": 2,
                    "y_tiles": 1
                  },
                  "7": {
                    "format": "png",
                    "pixel_ratio": 128,
                    "tile_height": 2048,
                    "tile_width": 2048,
                    "x_tiles": 1,
                    "y_tiles": 1
                  }
                },
                "width": 161717
              },
              "size_bytes": 0,
              "slot_name": "1cb65790-1e55-cb67-501d-a48b8ca05535.e47f119",
              "streamable": false,
              "total_sections": 1,
              "type": "image",
              "upload_id": "12b90921-95b7-4a1c-8d0e-5ed368ca661a"
            }
          ],
          "status": "new",
          "tags": [],
          "updated_at": "2023-07-18T01:56:05.801600Z",
          "uploads": [],
          "workflow_status": "new"


    }
        "#;

        let ser_item: DatasetItemV2 = serde_json::from_str(contents).unwrap();

        assert_eq!(ser_item.status, Some(DatasetItemStatus::New));
        assert_eq!(ser_item.dataset_id, Some(657106));
        assert_eq!(ser_item.slots.len(), 1);
        let levels = &ser_item
            .slots
            .first()
            .as_ref()
            .expect("Expected at least one slot")
            .as_ref()
            .expect("Expected metadata")
            .metadata
            .as_ref()
            .expect("Expected levels")
            .clone()
            .levels
            .expect("Expected levels");
        assert_eq!(levels.image_levels.len(), 8);
        assert_eq!(
            levels.image_levels.get(&0).unwrap().format,
            "png".to_string()
        );
    }
}
