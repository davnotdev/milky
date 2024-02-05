use milky::*;

struct Tray {}

struct Bird {
    man: RowMan<Bird>,
    transforms: Row<Transform2>,
    sprites: Row<Sprite>,
}

impl System for Bird {
    type Tray = Tray;
    type Planets = RuntimePlanets;

    fn new(_planets: &Self::Planets, _tray: &Self::Tray) -> Self {
        let mut man = RowMan::new();
        let mut transforms = Row::new(&mut man);
        let mut sprites = Row::new(&mut man);

        transforms.insert(Transform2 {
            position: glm::vec2(0.0, 0.0),
            scale: glm::vec2(1.0, 1.0),
            rotation: 0.0,
        });
        sprites.insert(Sprite {
            visible: true,
            color: glm::vec3(1.0, 0.0, 0.0),
        });
        let _ = man.insert();

        Self {
            transforms,
            sprites,
            man,
        }
    }

    fn run(&mut self, planets: &Self::Planets, _tray: &Self::Tray) {
        planets.rendering.get_ev().send(RenderSprites {
            sprites: self.sprites.get().to_vec(),
            transforms: self.transforms.get().to_vec(),
        });
    }
}

fn main() {
    let mut rt = Runtime::new(Tray {});
    rt.add_sys::<Bird>();
    rt.run();
}
