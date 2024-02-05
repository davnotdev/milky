use super::*;
use mime::Mime;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::{
    any::TypeId,
    collections::HashMap,
    fs,
    path::Path,
    sync::mpsc::{channel, Receiver},
};

pub struct FileAsset {
    pub data: Vec<u8>,
    pub mime: Vec<Mime>,
    pub path: String,
}

pub type AssetId = TypeId;

pub struct IOPlanet {
    watcher: RecommendedWatcher,
    assets: HashMap<TypeId, FileAsset>,
    watcher_receiver: Receiver<String>,
}

//  TODO: check implications
unsafe impl Send for IOPlanet {}
unsafe impl Sync for IOPlanet {}

impl IOPlanet {
    pub fn new() -> Self {
        let (tx, rx) = channel();
        //  TODO: Handle unwrap
        let watcher = notify::recommended_watcher(move |res: Result<Event, _>| match res {
            Ok(ev) => {
                for path in ev.paths {
                    //  TODO: Handle unwrap
                    tx.send(path.to_str().unwrap().to_owned()).unwrap();
                }
            }
            //  TODO: Handle error
            Err(e) => println!("watch error: {:?}", e),
        })
        .unwrap();

        IOPlanet {
            watcher,
            assets: HashMap::new(),
            watcher_receiver: rx,
        }
    }

    pub fn update(&mut self) {
        while let Ok(path) = self.watcher_receiver.try_recv() {
            //  TODO: Handle unwrap
            self.watcher.unwatch(Path::new(&path)).unwrap();
            let Some((asset_id, _)) = self.assets.iter().find(|(_, v)| v.path == path) else {
                //  TODO: Handle fail
                panic!()
            };
            let asset_id = *asset_id;
            self.assets.remove(&asset_id);
            self.load_asset_inner(asset_id, &path);
        }
    }

    pub fn get_asset(&self, asset_id: AssetId) -> Option<&FileAsset> {
        self.assets.get(&asset_id)
    }

    fn load_asset_inner(&mut self, asset_id: AssetId, path: &str) -> AssetId {
        //  TODO: Consider async system?
        //  TODO: Handle unwrap
        let data = fs::read(path).unwrap();
        let mime = mime_guess::from_path(path).iter().collect();

        //  TODO: Handle unwrap
        self.watcher
            .watch(Path::new(path), RecursiveMode::NonRecursive)
            .unwrap();

        self.assets.insert(
            asset_id,
            FileAsset {
                data,
                mime,
                path: path.to_owned(),
            },
        );
        asset_id
    }

    pub fn load_asset<A: 'static>(&mut self, path: &str) -> AssetId {
        let asset_id = TypeId::of::<A>();
        self.load_asset_inner(asset_id, path)
    }
}
