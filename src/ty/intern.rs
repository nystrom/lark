use crate::ty::debug::TyDebugContext;
use crate::ty::Generic;
use crate::ty::InferVar;
use crate::ty::{Base, BaseData, BaseKind};
use crate::ty::{Generics, GenericsData};
use crate::ty::{Perm, PermData};
use indexed_vec::{Idx, IndexVec};
use rustc_hash::FxHashMap;
use std::cell::RefCell;
use std::hash::Hash;
use std::rc::Rc;

#[derive(Debug)]
crate struct Interner<Key, Data>
where
    Key: Copy + Idx,
    Data: Clone + Hash + Eq,
{
    vec: IndexVec<Key, Data>,
    map: FxHashMap<Data, Key>,
}

impl<Key, Data> Interner<Key, Data>
where
    Key: Copy + Idx,
    Data: Clone + Hash + Eq,
{
    fn new() -> Self {
        Self {
            vec: IndexVec::default(),
            map: FxHashMap::default(),
        }
    }

    fn intern(&mut self, data: Data) -> Key {
        let Interner { vec, map } = self;
        map.entry(data.clone())
            .or_insert_with(|| vec.push(data))
            .clone()
    }
}

/// The "type context" is a global resource that interns types and
/// other type-related things. Types are allocates in the various
/// global arenas so that they can be freely copied around.
#[derive(Clone)]
crate struct TyInterners {
    data: Rc<TyInternersData>,
}

struct TyInternersData {
    perms: RefCell<Interner<Perm, PermData>>,
    bases: RefCell<Interner<Base, BaseData>>,
    generics: RefCell<Interner<Generics, GenericsData>>,
    common: Common,
}

crate struct Common {
    crate empty_generics: Generics,
    crate own: Perm,
}

crate trait Interners {
    fn interners(&self) -> &TyInterners;

    fn intern<D>(&self, data: D) -> D::Key
    where
        D: Intern,
    {
        data.intern(self.interners())
    }

    fn untern<K>(&self, key: K) -> K::Data
    where
        K: Untern,
    {
        key.untern(self.interners())
    }

    fn common(&self) -> &Common {
        &self.interners().data.common
    }

    fn intern_generics(&self, iter: impl Iterator<Item = Generic>) -> Generics {
        let generics_data = GenericsData {
            elements: Rc::new(iter.collect()),
        };
        self.intern(generics_data)
    }

    fn intern_base_var(&self, var: InferVar) -> Base {
        let data = BaseData {
            kind: BaseKind::Infer { var },
            generics: self.common().empty_generics,
        };
        self.intern(data)
    }
}

impl TyInterners {
    crate fn new() -> Self {
        let mut perms = Interner::new();
        let bases = Interner::new();
        let mut generics = Interner::new();

        let common = Common {
            own: perms.intern(PermData::Own),
            empty_generics: generics.intern(GenericsData {
                elements: Rc::new(vec![]),
            }),
        };

        TyInterners {
            data: Rc::new(TyInternersData {
                perms: RefCell::new(perms),
                bases: RefCell::new(bases),
                generics: RefCell::new(generics),
                common,
            }),
        }
    }
}

impl Interners for TyInterners {
    fn interners(&self) -> &TyInterners {
        self
    }
}

crate trait Intern: Clone {
    type Key;

    fn intern(self, interner: &TyInterners) -> Self::Key;
}

impl<T> Intern for &T
where
    T: Intern,
{
    type Key = T::Key;

    fn intern(self, interner: &TyInterners) -> Self::Key {
        self.clone().intern(interner)
    }
}

crate trait Untern: Clone {
    type Data;

    fn untern(self, interner: &TyInterners) -> Self::Data;
}

impl<T> Untern for &T
where
    T: Untern,
{
    type Data = T::Data;

    fn untern(self, interner: &TyInterners) -> Self::Data {
        self.clone().untern(interner)
    }
}

macro_rules! intern_ty {
    ($field:ident, $key:ty, $data:ty) => {
        impl Intern for $data {
            type Key = $key;

            fn intern(self, interner: &TyInterners) -> $key {
                interner.data.$field.borrow_mut().intern(self)
            }
        }

        impl Untern for $key {
            type Data = $data;

            fn untern(self, interner: &TyInterners) -> $data {
                interner.data.$field.borrow().vec[self].clone()
            }
        }
    };
}

intern_ty!(bases, Base, BaseData);
intern_ty!(perms, Perm, PermData);
intern_ty!(generics, Generics, GenericsData);

impl TyDebugContext for TyInterners {}