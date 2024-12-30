#[derive(PartialEq, Clone)]
pub enum BlockType {
    Profile,
    Repos,
    Search,
    Info,
    Commits,
    CommitInfo,
    SearchUser,
    SearchRepo,
    Default,
}

#[derive(PartialEq, Clone)]
pub enum BlockState {
    Default,
}

pub fn block_type(b_i: u8) -> BlockType {
    match b_i {
        0 => BlockType::Profile,
        1 => BlockType::Repos,
        2 => BlockType::Search,
        3 => BlockType::Info,
        4 => BlockType::Commits,
        5 => BlockType::CommitInfo,
        6 => BlockType::SearchUser,
        7 => BlockType::SearchRepo,
        _ => BlockType::Default,
    }
}

pub fn block_type_to_u8(block_type: BlockType) -> u8 {
    return block_type as u8;
}

pub struct BlockPos {
    pub col: usize,
    pub row: usize,
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
    state: BlockState,
    pos: BlockPos,
    sublayout: Option<TuiLayout>,
}

impl TuiBlock {
    fn new(b_type: BlockType, pos: BlockPos) -> Self {
        Self {
            b_type,
            state: BlockState::Default,
            pos,
            sublayout: None,
        }
    }

    pub fn block_type(&self) -> BlockType {
        return self.b_type.clone();
    }

    pub fn block_state(&self) -> BlockState {
        return self.state.clone();
    }

    pub fn set_state(&mut self, state: BlockState) {
        self.state = state;
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
    }

    pub fn active_block_pos(&self) -> &BlockPos {
        return &self.active;
    }

    pub fn active_block(&mut self) -> &mut TuiBlock {
        if self.sublayout_active && self.active_sublayout().is_some() {
            return self.active_sublayout().as_mut().unwrap().active_block();
        } else {
            return &mut self.blocks[self.active.col][self.active.row];
        }
    }

    pub fn active_sublayout(&mut self) -> &mut Option<TuiLayout> {
        return &mut self.blocks[self.active.col][self.active.row].sublayout;
    }

    pub fn next_block(&mut self) {
        match self.sublayout_active {
            true => {
                if let Some(sl) = self.active_sublayout() {
                    sl.next_block();
                }
            }
            false => {
                self.active.row = (self.active.row + 1) % self.blocks[self.active.col].len();
            }
        }
    }

    pub fn prev_block(&mut self) {
        match self.sublayout_active {
            true => {
                if let Some(sl) = self.active_sublayout() {
                    sl.prev_block();
                }
            }
            false => {
                if self.active.row == 0 {
                    self.active.row = self.blocks[self.active.col].len() - 1;
                } else {
                    self.active.row = (self.active.row - 1 + self.blocks[self.active.col].len())
                        % self.blocks[self.active.col].len();
                }
            }
        }
    }

    pub fn next_col(&mut self) {
        self.active.col = (self.active.col + 1) % self.blocks.len();
        self.active.row = self.active.row % self.blocks[self.active.col].len();
    }

    pub fn prev_col(&mut self) {
        if self.active.col == 0 {
            return;
        }
        self.active.col = (self.active.col - 1 + self.blocks.len()) % self.blocks.len();
        self.active.row = self.active.row % self.blocks[self.active.col].len();
    }

    pub fn select_layout(&mut self) {
        if self.active_block().sublayout.is_some() {
            self.sublayout_active = true;
        }
    }

    pub fn unselect_layout(&mut self) -> bool {
        if self.sublayout_active {
            self.sublayout_active = false;
            return true;
        }
        return false;
    }
}
