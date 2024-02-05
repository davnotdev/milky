use super::*;
use std::{cell::RefCell, marker::PhantomData, rc::Rc};

#[derive(Clone, Copy, PartialEq, Eq)]
enum RowStamp {
    PendingInsert { insert_index: usize },
    PendingRemove,
    Clear,
    Dropped,
}

pub struct Row<T> {
    datas: Vec<T>,
    rowman_stamp: Rc<RefCell<RowStamp>>,
}

impl<T> Row<T> {
    pub fn new<D>(rowman: &mut RowMan<D>) -> Self {
        let rowman_stamp = Rc::new(RefCell::new(RowStamp::Clear));
        rowman.register_row_stamp(Rc::clone(&rowman_stamp));
        Self {
            datas: vec![],
            rowman_stamp,
        }
    }

    pub fn insert(&mut self, data: T) {
        self.datas.push(data);
        *self.rowman_stamp.borrow_mut() = RowStamp::PendingInsert {
            insert_index: self.datas.len() - 1,
        };
    }

    //  TODO: better api needed
    pub fn get(&self) -> &Vec<T> {
        &self.datas
    }
}

impl<T> Drop for Row<T> {
    fn drop(&mut self) {
        *self.rowman_stamp.borrow_mut() = RowStamp::Dropped;
    }
}

//  TODO: consider implications
unsafe impl<T> Send for Row<T> {}
unsafe impl<T> Sync for Row<T> {}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Entity<T> {
    id: usize,
    gen: usize,
    _phantom: PhantomData<T>,
}

pub struct RowMan<D> {
    entities: Vec<(usize, bool)>,
    stamps: Vec<Rc<RefCell<RowStamp>>>,
    _phantom: PhantomData<D>,
}

impl<D> RowMan<D> {
    pub fn new() -> Self {
        Self {
            entities: vec![],
            stamps: vec![],
            _phantom: PhantomData,
        }
    }

    fn register_row_stamp(&mut self, stamp: Rc<RefCell<RowStamp>>) {
        self.stamps.push(stamp);
    }

    pub fn insert(&mut self) -> Entity<D> {
        //  Because of the checking we do, data_len should be identical for all `PendingInsert`s
        let mut entity_idx = 0;

        for stamp in self.stamps.iter() {
            let mut stamp = stamp.borrow_mut();
            if std::mem::discriminant(&*stamp)
                != std::mem::discriminant(&RowStamp::PendingInsert { insert_index: 0 })
                && *stamp != RowStamp::Dropped
            {
                panic!("Row pending insert");
            } else if *stamp != RowStamp::Dropped {
                let RowStamp::PendingInsert { insert_index } = *stamp else {
                    unreachable!()
                };
                entity_idx = insert_index;
                *stamp = RowStamp::Clear;
            }
        }
        self.insert_inner(entity_idx)
    }

    pub fn remove(&mut self, entity: Entity<D>) {
        for stamp in self.stamps.iter() {
            let mut stamp = stamp.borrow_mut();
            if *stamp != RowStamp::PendingRemove && *stamp != RowStamp::Dropped {
                panic!("Row pending remove");
            } else if *stamp != RowStamp::Dropped {
                *stamp = RowStamp::Clear;
            }
        }
        self.remove_inner(entity);
    }

    fn insert_inner(&mut self, idx: usize) -> Entity<D> {
        if idx >= self.entities.len() {
            self.entities.resize(self.entities.len() + idx + 1, (0, false));
        }

        let Some((generation, exists)) = self.entities.get_mut(idx) else {
            unreachable!("We just resized tho?")
        };
        if *exists {
            unreachable!("Entity already taken??")
        }
        *generation += 1;
        *exists = true;
        Entity {
            id: idx,
            gen: *generation,
            _phantom: PhantomData,
        }
    }

    fn remove_inner(&mut self, entity: Entity<D>) {
        let (generation, exists) = self.entities.get_mut(entity.id).unwrap();
        if *generation != entity.gen {
            panic!("Wrong generation buddy");
        }
        *exists = false;
    }
}

//  TODO: consider implications
unsafe impl<D> Send for RowMan<D> {}
unsafe impl<D> Sync for RowMan<D> {}
