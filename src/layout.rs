#[derive(PartialEq, Clone)]
pub enum BlockType {
    Profile,
    Repos,
    Search,
    Info,
    Commits,
    SearchResults,
    SearchUser,
    SearchRepo,
    Default,
}

pub fn block_type(b_i: u8) -> BlockType {
    match b_i {
        0 => BlockType::Profile,
        1 => BlockType::Repos,
        2 => BlockType::Search,
        3 => BlockType::Info,
        4 => BlockType::Commits,
        5 => BlockType::SearchResults,
        6 => BlockType::SearchUser,
        7 => BlockType::SearchRepo,
        _ => BlockType::Default,
    }
}

pub fn block_type_to_u8(block_type: BlockType) -> u8 {
    return block_type as u8;
}

pub fn create_test_layout(layout: &mut TuiLayout) {
    layout.add_col();
    layout.add_col();
    layout.add_block(BlockType::Profile, 0);
    layout.add_block(BlockType::Repos, 0);
    let sub_lo = layout.add_layout(BlockType::Search, 0);
    sub_lo.add_col();
    sub_lo.add_block(BlockType::SearchUser, 0);
    sub_lo.add_block(BlockType::SearchRepo, 0);
    layout.add_block(BlockType::Info, 1);
    layout.add_block(BlockType::Commits, 1);
    layout.add_block(BlockType::SearchResults, 1);
}

struct BlockPos {
    col: usize,
    row: usize,
}

impl BlockPos {
    fn new(col: usize, row: usize) -> Self {
        Self { col, row }
    }

    fn default() -> Self {
        Self { col: 0, row: 0 }
    }
}

pub struct TuiBlock {
    b_type: BlockType,
    pos: BlockPos,
    sublayout: Option<TuiLayout>,
}

impl TuiBlock {
    fn new(b_type: BlockType, pos: BlockPos) -> Self {
        Self {
            b_type,
            pos,
            sublayout: None,
        }
    }

    pub fn block_type(&self) -> BlockType {
        return self.b_type.clone();
    }
}

pub struct TuiLayout {
    cols: usize,
    rows: usize,
    blocks: Vec<Vec<TuiBlock>>,
    active: BlockPos,
    sublayout_active: bool,
}

impl TuiLayout {
    pub fn new() -> Self {
        Self {
            cols: 0,
            rows: 0, // TODO : Needed?
            blocks: Vec::new(),
            active: BlockPos::default(),
            sublayout_active: false,
        }
    }

    pub fn print_status(&self) -> String {
        return format!("Row: {}, Col: {}", self.active.row, self.active.col);
    }

    pub fn add_col(&mut self) {
        self.blocks.push(Vec::new());
        self.cols += 1;
    }

    pub fn add_block(&mut self, block_type: BlockType, col: usize) {
        if col > self.blocks.len() {
            return;
        }
        let pos = BlockPos::new(col, self.blocks[col].len());
        let block = TuiBlock::new(block_type, pos);
        self.blocks[col].push(block);
    }

    pub fn add_layout(&mut self, block_type: BlockType, col: usize) -> &mut TuiLayout {
        let len = self.blocks[col].len();
        let pos = BlockPos::new(col, len);
        let mut block = TuiBlock::new(block_type, pos);
        block.sublayout = Some(TuiLayout::new());
        self.blocks[col].push(block);
        return self.blocks[col]
            .last_mut()
            .unwrap()
            .sublayout
            .as_mut()
            .unwrap();
        // BlockPos::new(col, len)
    }

    pub fn active_block(&self) -> &TuiBlock {
        return &self.blocks[self.active.col][self.active.row];
    }

    pub fn next_block(&mut self) {
        self.active.row = (self.active.row + 1) % self.blocks[self.active.col].len();
    }

    pub fn prev_block(&mut self) {
        if self.active.row == 0 {
            self.active.row = self.blocks[self.active.col].len() - 1;
        } else {
            self.active.row = (self.active.row - 1 + self.blocks[self.active.col].len())
                % self.blocks[self.active.col].len();
        }
    }

    pub fn next_col(&mut self) {
        self.active.col = (self.active.col + 1) % self.blocks.len();
        self.active.row = self.active.row % self.blocks[self.active.col].len();
    }

    pub fn prev_col(&mut self) {
        self.active.col = (self.active.col - 1 + self.blocks.len()) % self.blocks.len();
        self.active.row = self.active.row % self.blocks[self.active.col].len();
    }

    pub fn select_layout(&mut self) {
        if self.active_block().sublayout.is_some() {
            self.sublayout_active = true;
        }
    }

    pub fn unselect_layout(&mut self) {
        self.sublayout_active = false;
    }
}
