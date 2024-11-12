#[derive(PartialEq)]
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

struct TuiBlock {
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
}

struct TuiLayout {
    cols: usize,
    rows: usize,
    blocks: Vec<Vec<TuiBlock>>,
    active: BlockPos,
    sublayout_active: bool,
}

impl TuiLayout {
    fn new() -> Self {
        Self {
            cols: 0,
            rows: 0, // TODO : Needed?
            blocks: Vec::new(),
            active: BlockPos::default(),
            sublayout_active: false,
        }
    }

    fn add_col(&mut self) {
        self.blocks.push(Vec::new());
        self.cols += 1;
    }

    fn add_block(&mut self, block_type: BlockType, col: usize) {
        if col > self.blocks.len() {
            return;
        }
        let pos = BlockPos::new(col, self.blocks[col].len());
        let block = TuiBlock::new(block_type, pos);
        self.blocks[col].push(block);
    }

    fn add_layout(&mut self, block_type: BlockType, col: usize) -> BlockPos {
        let len = self.blocks[col].len();
        let pos = BlockPos::new(col, len);
        let mut block = TuiBlock::new(block_type, pos);
        block.sublayout = Some(TuiLayout::new());
        self.blocks[col].push(block);
        BlockPos::new(col, len)
    }

    // TODO: delete
    fn add_block_deprecated(&mut self, block: TuiBlock, col: usize) {
        self.blocks[col].push(block);
    }

    fn active_block(&self) -> &TuiBlock {
        return &self.blocks[self.active.col][self.active.row];
    }

    fn next_block(&mut self) {
        self.active.row = (self.active.row + 1) % self.blocks[self.active.col].len();
    }

    fn prev_block(&mut self) {
        self.active.row = (self.active.row - 1 + self.blocks[self.active.col].len())
            % self.blocks[self.active.col].len();
    }

    fn next_col(&mut self) {
        self.active.col = (self.active.col + 1) % self.blocks.len();
    }

    fn prev_col(&mut self) {
        self.active.col = (self.active.col - 1 + self.blocks.len()) % self.blocks.len();
    }

    fn select_layout(&mut self) {
        if self.active_block().sublayout.is_some() {
            self.sublayout_active = true;
        }
    }

    fn unselect_layout(&mut self) {
        self.sublayout_active = false;
    }
}
