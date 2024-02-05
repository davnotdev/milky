pub trait System: Send + Sync {
    type Tray;
    type Planets;

    fn new(planets: &Self::Planets, tray: &Self::Tray) -> Self
    where
        Self: Sized;
    fn run(&mut self, planets: &Self::Planets, tray: &Self::Tray);
}
