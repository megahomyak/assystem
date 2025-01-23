use std::io::{Read, Seek, SeekFrom, Write};
/*
Memory structure:
* File format name (['A', 'S', 'S', ' ', 'v', '1', '\0'])
* Root node 0 branch index (may be null)
* Root node 1 branch index (may be null)
* Root node content index (may be null)
* First block flags
* First block length (non-null)
* Heap (with blocks: [block flags, first block length (non-null), prev block index (non-null)])

Flags: is_free, is_end
*/

const IS_FREE_MASK: u8 = 0b1000_0000u8;
const IS_END_MASK: u8 = 0b0100_0000u8;
const IS_PADDING_MASK: u8 = 0b0010_0000u8;
const FILE_FORMAT_NAME_LENGTH: u8 = 7;
const FILE_FORMAT_NAME: [u8; FILE_FORMAT_NAME_LENGTH as usize] =
    [b'A', b'S', b'S', b' ', b'v', b'1', b'\0'];
mod offsets {
    pub const FILE_FORMAT_NAME: u8 = 0;
    pub const ROOT_NODE_0_BRANCH_INDEX: u8 = FILE_FORMAT_NAME + super::FILE_FORMAT_NAME_LENGTH;
    pub const ROOT_NODE_1_BRANCH_INDEX: u8 = ROOT_NODE_0_BRANCH_INDEX + 8;
    pub const ROOT_NODE_CONTENT_INDEX: u8 = ROOT_NODE_1_BRANCH_INDEX + 8;
    pub const FIRST_BLOCK_FLAGS: u8 = ROOT_NODE_CONTENT_INDEX + 8;
    pub const AFTER_FIRST_BLOCK_INDEX: u8 = FIRST_BLOCK_FLAGS + 1;
    pub const HEAP: u8 = AFTER_FIRST_BLOCK_INDEX + 8;
}

mod sizes {
    pub const BLOCK_INDEX: u8 = 1;
    pub const BLOCK_LENGTH: u8 = 8;
    pub const BLOCK_PREV: u8 = 8;

    pub const BLOCK: u8 = BLOCK_INDEX + BLOCK_LENGTH + BLOCK_PREV;
}

type FileIndex = u64;

pub struct ASS {
    file: std::fs::File,
}
pub enum OpeningError {
    /// Not an ASS file of the needed version, unfortunately.
    Assless(),
    IO(std::io::Error),
}
impl ASS {
    fn read<const N: usize>(&mut self) -> [u8; N] {
        let mut result = [0u8; N];
        self.file.read_exact(&mut result).unwrap();
        result
    }
    fn write(&mut self, data: &[u8]) {
        self.file.write_all(data).unwrap();
    }

    fn write_index(&mut self, index: u64) {
        self.write(&index.to_be_bytes());
    }
    fn write_flags(&mut self, index: u8) {
        self.write(&[index]);
    }
    fn read_u8(&mut self) -> u8 {
        self.read::<1>()[0]
    }
    fn read_u64(&mut self) -> u64 {
        u64::from_be_bytes(self.read::<8>())
    }
    fn alloc(&mut self, amount: u64) -> FileIndex {
        let mut block_beginning = offsets::FIRST_BLOCK_FLAGS as u64;
        self.file
            .seek(SeekFrom::Start(block_beginning));
        loop {
            let flags = self.read_u8();
            if flags & IS_PADDING_MASK != 0 {
                continue;
            }
            let length = self.read_u64();
            if self.tell() != offsets::HEAP as u64 {
                self.file.seek(SeekFrom::Current(8));
            }
            if flags & IS_FREE_MASK != 0 {
                if length == amount {
                    self.file.seek(SeekFrom::Start(block_beginning)).unwrap();
                    self.write_flags(flags & !IS_FREE_MASK);
                    self.file.seek(SeekFrom::Current(8)).unwrap();
                    self.write
                    // repurpose the block
                } else if amount < length {
                    // add a block inside a block.
                    // add a second header in-between if possible,
                    // add padding in-between otherwise
                }
            }
            if flags & IS_END_MASK != 0 {
                if flags & IS_FREE_MASK != 0 {
                    // extend current block
                } else {
                    self.file
                        .seek(SeekFrom::Current(length.try_into().unwrap()));
                    // go after current block's data and make a new block
                }
                return;
            }
            block_beginning = self.file
                .seek(SeekFrom::Current(length.try_into().unwrap()));
        }
    }
    /// The index should be valid to prevent database breakage
    fn dealloc(&mut self, index: FileIndex) {}
    pub fn get(&mut self, key: &[u8]) -> Vec<u8> {
        if content_index == 1 {
            return Vec::new();
        }
    }
    pub fn set(&mut self, key: &[u8], value: &[u8]) {
        if value.len() == 0 {
            // set "1" as address instead of allocating
        }
    }
    pub fn open(path: impl AsRef<std::path::Path>) -> Result<Self, OpeningError> {
        // =()=
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&path);
        match file {
            Ok(mut file) => {
                // check the file header
                let mut existing_header = [0u8; FILE_FORMAT_NAME.len()];
                file.read_exact(&mut existing_header)
                    .map_err(|_| OpeningError::Assless())?;
                if existing_header
                    .iter()
                    .zip(FILE_FORMAT_NAME.iter())
                    .all(|(a, b)| a == b)
                {
                    // all match, we're fine
                    Ok(Self { file })
                } else {
                    Err(OpeningError::Assless())
                }
            }
            Err(err) => {
                if let std::io::ErrorKind::NotFound = err.kind() {
                    // create, fill, done
                    let file = std::fs::OpenOptions::new()
                        .read(true)
                        .write(true)
                        .open(&path)
                        .map_err(|err| OpeningError::IO(err))?;
                    let mut this = Self { file };
                    this.write(&FILE_FORMAT_NAME);
                    this.write_index(0);
                    this.write_index(0);
                    this.write_index(0);
                    this.write_flags(IS_FREE_MASK | IS_END_MASK);
                    this.write_index(0);
                    Ok(this)
                } else {
                    Err(OpeningError::IO(err))
                }
            }
        }
    }
}

fn main() {
    println!("Hello, world!");
}
