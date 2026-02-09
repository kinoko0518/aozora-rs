use itertools::Itertools;

pub fn get_xhtml_filename(id: usize) -> String {
    format!("xhtml{:>04}.xhtml", id)
}

#[derive(Clone)]
pub struct CDepth {
    pub depth: [usize; 3],
}

impl CDepth {
    pub fn new() -> Self {
        Self { depth: [0, 0, 0] }
    }

    pub fn increament_a(&mut self) {
        self.depth[0] += 1;
        self.depth[1] = 0;
        self.depth[2] = 0;
    }

    pub fn increament_b(&mut self) {
        self.depth[1] += 1;
        self.depth[2] = 0;
    }

    pub fn increament_c(&mut self) {
        self.depth[2] += 1;
    }
}

pub struct Chapter {
    /// 連番で割り当てられるXHTML特有の番号です。
    pub xhtml_id: usize,
    /// 読者が実際に読む章の名前です。
    pub name: String,
    /// 1-2-3のような章の階層構造を保持します。
    pub depth: CDepth,
}

impl Chapter {
    pub fn get_id(&self) -> String {
        format!("chapter-{}", self.depth.depth.iter().join("-"))
    }

    pub fn get_nav(&self) -> String {
        format!("{}#{}", get_xhtml_filename(self.xhtml_id), self.get_id())
    }
}
