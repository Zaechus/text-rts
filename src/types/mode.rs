#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Mode {
    Select,
    Move,
    Attack,
    Hold,
    Build,
    Ctrl,
    Add,
}
