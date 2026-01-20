use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::RwLock,
};

#[derive(Default)]
pub struct GlobalDependencies {
    #[allow(clippy::type_complexity)]
    pub dependencies: HashMap<(TypeId, Option<String>), &'static dyn Any>,
}

thread_local! {
    static GLOBAL_DEPENDENCIES: RwLock<GlobalDependencies> = RwLock::new(GlobalDependencies::default())
}

pub fn get_global_dep<T: Any>(discriminator: Option<&str>) -> Option<&'static RwLock<T>> {
    GLOBAL_DEPENDENCIES.with(move |dependencies| {
        dependencies.clear_poison();
        let lock: &'static RwLock<T> = (*dependencies
            .read()
            .unwrap()
            .dependencies
            .get(&(TypeId::of::<T>(), discriminator.map(|s| s.to_owned())))?)
        .downcast_ref()
        .unwrap();
        lock.clear_poison();

        Some(lock)
    })
}

pub fn set_global_dep<T: Any>(value: T, discriminator: Option<&str>) -> Option<T> {
    if let Some(lock) = get_global_dep::<T>(discriminator) {
        lock.clear_poison();
        Some(lock.replace(value).unwrap())
    } else {
        GLOBAL_DEPENDENCIES.with(move |dependencies| {
            dependencies.clear_poison();

            dependencies.write().unwrap().dependencies.insert(
                (TypeId::of::<T>(), discriminator.map(|s| s.to_owned())),
                Box::leak(Box::new(RwLock::new(value))),
            );
        });
        None
    }
}

#[macro_export]
macro_rules! global_dep {
    ($t:ty) => {
        hydrogen::core::global_dependency::get_global_dep::<$t>(None)
            .unwrap()
            .try_read()
            .unwrap()
    };
    ($t:ty, $disc:expr) => {
        hydrogen::core::global_dependency::get_global_dep::<$t>(Some($disc))
            .unwrap()
            .try_read()
            .unwrap()
    };
    (mut $t:ty) => {
        hydrogen::core::global_dependency::get_global_dep::<$t>(None)
            .unwrap()
            .try_write()
            .unwrap()
    };
    (mut $t:ty, $disc:expr) => {
        hydrogen::core::global_dependency::get_global_dep::<$t>(Some($disc))
            .unwrap()
            .try_write()
            .unwrap()
    };
}

#[macro_export]
macro_rules! try_global_dep {
    ($t:ty) => {
        hydrogen::core::global_dependency::get_global_dep::<$t>(None)
            .map(|lock| lock.try_read().unwrap())
    };
    ($t:ty, $disc:expr) => {
        hydrogen::core::global_dependency::get_global_dep::<$t>(Some($disc))
            .map(|lock| lock.try_read().unwrap())
    };
    (mut $t:ty) => {
        hydrogen::core::global_dependency::get_global_dep::<$t>(None)
            .map(|lock| lock.try_write().unwrap())
    };
    (mut $t:ty, $disc:expr) => {
        hydrogen::core::global_dependency::get_global_dep::<$t>(Some($disc))
            .map(|lock| lock.try_write().unwrap())
    };
}

pub use {global_dep, try_global_dep};
