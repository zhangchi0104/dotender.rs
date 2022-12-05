use super::Config;

pub trait Command {
    fn run(&mut self, cfg: &mut Config);
}
