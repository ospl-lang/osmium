use ospl_common::ast::frame::RuntimeFrame;

#[cfg(debug_assertions)]
use crate::debug::DbgMark;

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
            base: self.data.len(),  // this makes it work??
            own: 0,
            size: 0,
        });
        #[cfg(debug_assertions)] let _ = DbgMark::new(format!("pushed isolated: {:?}", self.top_meta()));
    }

    pub fn push_parental(&mut self) {
        let top = self.top_meta();
        self.frames.push(Frame {
            own: self.data.len(),
            size: top.size,
            base: top.base,
        });
        #[cfg(debug_assertions)] let _ = DbgMark::new(format!("pushed parental: {:?}", self.top_meta()));
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
            own: 0,
        });
        self.data.extend(frame.indexes);
    }

    pub fn end(&mut self) {
        #[cfg(debug_assertions)] let _ = DbgMark::new(format!("before frame end: {self:?}"));
        let Some(f) = self.frames.pop() else {
            return;
        };

        // Restore stack to frame start
        self.data.truncate(f.base + f.own);
        #[cfg(debug_assertions)] let _ = DbgMark::new(format!("after frame end (truncated to {}): {self:?}", f.base + f.own));
    }

    pub fn pop(&mut self) -> RuntimeFrame {
        #[cfg(debug_assertions)] let _ = DbgMark::new(format!("before frame pop: {self:?}"));
        let f = self.frames.pop().unwrap();

        let mut r = RuntimeFrame::default();

        // Extract only what belongs to this frame
        r.indexes = self.data.split_off(f.base + f.own);
        #[cfg(debug_assertions)] let _ = DbgMark::new(format!("after frame pop (truncated to {}): {self:?}", f.base + f.own));
        return r
    }
}

#[derive(Debug)]
pub struct Stack {
    pub data: Vec<usize>,
    pub frames: Vec<Frame>
}

impl Default for Stack {
    fn default() -> Self {
        let mut list = Vec::new();
        list.push(Frame {
            base: 0,
            size: 0,
            own: 0,
        });

        return Self {
            // data: ArrayVec::new(),
            data: Vec::new(),
            frames: list
        }
    }
}