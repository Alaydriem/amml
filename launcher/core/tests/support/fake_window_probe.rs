use amml_launcher_core::WindowProbe;
use std::cell::Cell;
use std::rc::Rc;

// Says there is a window once it was asked `after` times. Clones share the counter.
#[derive(Clone)]
pub struct FakeWindowProbe {
    after: usize,
    asked: Rc<Cell<usize>>,
}

impl FakeWindowProbe {
    pub fn after(after: usize) -> FakeWindowProbe {
        FakeWindowProbe { after, asked: Rc::new(Cell::new(0)) }
    }

    pub fn never() -> FakeWindowProbe {
        FakeWindowProbe::after(usize::MAX)
    }

    pub fn asked(&self) -> usize {
        self.asked.get()
    }
}

impl WindowProbe for FakeWindowProbe {
    fn has_window(&self, _pid: u32) -> bool {
        self.asked.set(self.asked.get() + 1);
        self.asked.get() >= self.after
    }
}
