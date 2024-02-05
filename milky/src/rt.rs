use super::{planets::WindowPlanet, *};
use rayon::prelude::*;

pub struct RuntimePlanets {
    pub window: WindowPlanet,
    pub rendering: RenderingPlanet,
    pub io: IOPlanet,
}

pub struct Runtime<T> {
    pub planets: RuntimePlanets,

    systems: Vec<Box<dyn System<Tray = T, Planets = RuntimePlanets>>>,
    tray: T,
}

impl<T> Runtime<T>
where
    T: Send + Sync,
{
    pub fn new(tray: T) -> Self {
        let io = IOPlanet::new();
        let window = WindowPlanet::new();
        let rendering = RenderingPlanet::new(&window);

        Self {
            tray,
            planets: RuntimePlanets {
                io,
                window,
                rendering,
            },
            systems: vec![],
        }
    }

    pub fn add_sys<S: System<Tray = T, Planets = RuntimePlanets> + 'static>(&mut self) {
        self.systems
            .push(Box::new(S::new(&self.planets, &self.tray)))
    }

    pub fn tick(&mut self) {
        self.planets.io.update();
        self.planets.rendering.update(&self.planets.window);

        self.systems
            .par_iter_mut()
            .for_each(|sys| sys.run(&self.planets, &self.tray));
    }

    pub fn run(&mut self) {
        run_window_loop(self);
    }
}
