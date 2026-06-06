use arrayvec::ArrayVec;
use ospl_common::ast::frame::RuntimeFrame;

#[derive(Debug)]
pub struct Frame {
    /// Where does our inheritence begin?
    pub base: usize,

    /// Where does own ownership begin?
    pub own: usize,

    /// How many items are in the scope?
    pub size: usize,
}

impl Stack {
    pub fn push_isolated(&mut self) {
        self.frames.push(Frame {
            base: self.data.len(),
            own: self.data.len(),
            size: 0
        });
    }

    pub fn push_parental(&mut self) {
        let top = self.top_meta();
        self.frames.push(Frame {
            own: self.data.len(),
            ..*top
        });
    }

    pub fn top_meta(&self) -> &Frame {
        return self.frames.last().unwrap()
    }

    pub fn top_meta_mut(&mut self) -> &mut Frame {
        return self.frames.last_mut().unwrap()
    }

    pub fn top_indexes(&self) -> &[usize] {
        let t = self.top_meta();
        return &self.data[t.base..(t.base + t.size)];
    }

    pub fn top_indexes_mut(&mut self) -> &mut [usize] {
        let (base, size) = {
            let t = self.top_meta();
            (t.base, t.size)
        };
        return &mut self.data[base..(base + size)];
    }

    pub fn top_add_index(&mut self, abs: usize) {
        self.data.push(abs);
        self.top_meta_mut().size += 1;
    }

    pub fn push_frame_value(&mut self, frame: RuntimeFrame) {
        self.frames.push(Frame {
            base: self.data.len(),
            size: frame.indexes.len(),
            own: 0
        });
        self.data.extend(frame.indexes);
    }

    pub fn end(&mut self) {
        let Some(f) = self.frames.pop()
        else { return; };

        // let base = self.data.len() - f.size;
        // self.data.truncate(base);

        for _ in 0..f.size - f.own {
            self.data.pop();
        }
    }

    pub fn pop(&mut self) -> RuntimeFrame {
        let mut r = RuntimeFrame::default();
        let f = self.frames.pop().unwrap();

        for _ in 0..f.size - f.own {
            r.indexes.push(self.data.pop().unwrap());
        }

        return r
    }
}

#[derive(Debug)]
pub struct Stack {
    pub data: ArrayVec<usize, 8192>,
    pub frames: ArrayVec<Frame, 1024>,
}

impl Default for Stack {
    fn default() -> Self {
        let mut list = arrayvec::ArrayVec::new();
        list.push(Frame {
            base: 0,
            size: 0,
            own: 0
        });

        return Self {
            data: ArrayVec::new(),
            frames: list
        }
    }
}