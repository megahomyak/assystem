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
const FILE_FORMAT_NAME_LENGTH: u8 = 7;
const FILE_FORMAT_NAME: [u8; FILE_FORMAT_NAME_LENGTH as usize] =
    [b'A', b'S', b'S', b' ', b'v', b'1', b'\0'];
mod offsets {
    pub const FILE_FORMAT_NAME: u64 = 0;
    pub const ROOT_NODE_0_BRANCH_INDEX: u64 =
        FILE_FORMAT_NAME + super::FILE_FORMAT_NAME_LENGTH as u64;
    pub const ROOT_NODE_1_BRANCH_INDEX: u64 = ROOT_NODE_0_BRANCH_INDEX + 8;
    pub const ROOT_NODE_CONTENT_INDEX: u64 = ROOT_NODE_1_BRANCH_INDEX + 8;
    pub const FIRST_BLOCK_FLAGS: u64 = ROOT_NODE_CONTENT_INDEX + 8;
    pub const AFTER_FIRST_BLOCK_INDEX: u64 = FIRST_BLOCK_FLAGS + 1;
    pub const HEAP: u64 = AFTER_FIRST_BLOCK_INDEX + 8;
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
    fn tell(&mut self) -> u64 {
        self.file.seek(SeekFrom::Current(0)).unwrap()
    }
    fn seek(&mut self, index: u64) {
        self.file.seek(SeekFrom::Start(index)).unwrap();
    }
    fn alloc(&mut self, amount: u64) -> FileIndex {
        self.seek(offsets::FIRST_BLOCK_FLAGS);
        let mut flags = self.read_u8();
        let mut length = self.read_u64();
        loop {
            if flags & IS_FREE_MASK != 0 {
                if length == amount {
                    // repurpose the block
                } else if length <= amount + 8 + 8 + 1 {
                    // add a block inside a block
                }
            }
            if flags & IS_END_MASK != 0 {
                if flags & IS_FREE_MASK != 0 {
                    // extend current block (keep in mind the extra "prev" index)
                } else {
                    // go after current block's data (keep in mind the extra "prev" index) and make a new block
                }
                break;
            }
            self.seek(self.tell() + length);
            flags = self.read_u8();
            length = self.read_u64();
        }
    }
    /// The index should be valid to prevent database breakage
    fn dealloc(&mut self, index: FileIndex) {}
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
