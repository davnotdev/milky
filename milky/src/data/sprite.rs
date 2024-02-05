use super::*;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct Sprite {
    pub visible: bool,
    pub color: glm::Vec3,
}
