use glutin::{window::Window, ContextWrapper, PossiblyCurrent};

pub struct Smitten {
    context: ContextWrapper<PossiblyCurrent, Window>,
}
