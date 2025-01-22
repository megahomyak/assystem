const FILE_FORMAT_NAME_LENGTH: u8 = 7;
const FILE_FORMAT_NAME: [char; FILE_FORMAT_NAME_LENGTH as usize] = ['A', 'S', 'S', ' ', 'v', '1', '\0'];
type FileIndex = u64;
/*
Memory structure:
* File format name (['A', 'S', 'S', ' ', 'v', '1', '\0'])
* Root node 0 branch index (may be null)
* Root node 1 branch index (may be null)
* Root node content index (may be null)
* First block flags
* After first block index (non-null)
* Heap (with blocks: [block flags, after block index (non-null), prev block index (non-null)])
*/
pub fn write_index(file: std::fs::File, index: u64) {
    
}
mod sizes {
    pub const INDEX: u64 = 4;
    pub const FLAGS: u64 = 1;

    pub const BLOCK_FLAGS: u64 = FLAGS;
    pub const AFTER_BLOCK_INDEX: u64 = INDEX;
    pub const PREV_BLOCK_INDEX: u64 = INDEX;

    pub const FILE_FORMAT_NAME: u64 = super::FILE_FORMAT_NAME_LENGTH as u64;
    pub const ROOT_NODE_0_BRANCH_INDEX: u64 = INDEX;
    pub const ROOT_NODE_1_BRANCH_INDEX: u64 = INDEX;
    pub const ROOT_NODE_CONTENT_INDEX: u64 = INDEX;
    pub const FIRST_BLOCK_FLAGS: u64 = BLOCK_FLAGS;
    pub const AFTER_FIRST_BLOCK_INDEX: u64 = AFTER_BLOCK_INDEX;
}
mod indexes {
    use super::sizes;

    pub const BLOCK_FLAGS: u64 = 0;
    pub const AFTER_BLOCK_INDEX: u64 = BLOCK_FLAGS + sizes::BLOCK_FLAGS;
    pub const PREV_BLOCK_INDEX: u64 = AFTER_BLOCK_INDEX + sizes::AFTER_BLOCK_INDEX;

    pub const FILE_FORMAT_NAME: u64 = 0;
    pub const ROOT_NODE_0_BRANCH_INDEX: u64 = FILE_FORMAT_NAME + sizes::FILE_FORMAT_NAME;
    pub const ROOT_NODE_1_BRANCH_INDEX: u64 = ROOT_NODE_0_BRANCH_INDEX + sizes::ROOT_NODE_0_BRANCH_INDEX;
    pub const ROOT_NODE_CONTENT: u64 = ROOT_NODE_1_BRANCH_INDEX + sizes::ROOT_NODE_1_BRANCH_INDEX;
    pub const FIRST_BLOCK_FLAGS: u64 = ROOT_NODE_CONTENT + sizes::ROOT_NODE_CONTENT_INDEX;
    pub const AFTER_FIRST_BLOCK_INDEX: u64 = FIRST_BLOCK_FLAGS + sizes::FIRST_BLOCK_FLAGS;
    pub const HEAP_INDEX: u64 = AFTER_FIRST_BLOCK_INDEX + sizes::AFTER_FIRST_BLOCK_INDEX;
}
pub struct ASS {
    file: std::fs::File,
}
pub enum OpeningError {
    /// Not an ASS file of the needed version, unfortunately.
    Assless(),
    IO(std::io::Error),
}
impl ASS {
    fn allocate(&mut self, bytes_amount: u64) -> FileIndex {

    }
    fn deallocate(&mut self, area: FileIndex) {
    
    }
    pub fn open(path: impl AsRef<std::path::Path>) -> Result<Self, std::io::Error> {
        // =()=

    }
}

fn main() {
    println!("Hello, world!");
}
