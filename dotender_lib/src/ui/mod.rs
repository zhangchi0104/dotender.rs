pub trait StatefulUI<T> {
    fn update(&mut self, new_state: T);
}
pub mod install;
