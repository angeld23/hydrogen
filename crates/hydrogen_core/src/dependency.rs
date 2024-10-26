pub use hydrogen_core_proc_macro::DependencyProvider;

pub trait Dependency<D> {
    fn dep(&self) -> &D;
}

pub trait DependencyMut<D> {
    fn dep_mut(&mut self) -> &mut D;
}
