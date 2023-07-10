#[derive(Debug, Copy, Clone)]
pub enum Size {
    KiB(usize),
    MiB(usize),
    GiB(usize),
    TiB(usize),
}

impl Size {
    pub const fn bytes(self) -> usize {
        match self {
            Size::KiB(x) => x * 1024,
            Size::MiB(x) => x * Size::KiB(1024).bytes(),
            Size::GiB(x) => x * Size::MiB(1024).bytes(),
            Size::TiB(x) => x * Size::GiB(1024).bytes(),
        }
    }
}