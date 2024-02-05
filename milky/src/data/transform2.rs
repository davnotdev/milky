use super::*;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct Transform2 {
    pub position: glm::Vec2,
    pub scale: glm::Vec2,
    pub rotation: f32,
}
