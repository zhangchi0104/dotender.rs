use super::{Config, Error};

pub trait Command {
    fn run<'a>(&mut self, cfg: &'a mut Config) -> Result<(), Error<'a>>;
}
