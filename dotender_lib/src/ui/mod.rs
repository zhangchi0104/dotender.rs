pub trait StatefulUI<T> {
    fn update(&mut self, new_state: T);
    fn done_with_msg(&mut self, msg: &'static str);
}
pub mod install;
