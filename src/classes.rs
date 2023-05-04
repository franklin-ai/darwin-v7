use fake::Dummy;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize, Dummy, PartialEq, Eq)]
pub struct BoundingBox {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}
