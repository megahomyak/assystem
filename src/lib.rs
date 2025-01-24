use std::io::{Read, Seek, SeekFrom, Write};
/*
Memory structure:
* Root node 0 branch index (may be null)
* Root node 1 branch index (may be null)
* Root node content index (may be null)
* Blocks

Blocks order: free -> taken -> free -> taken -> ... -> taken -> free
Block structure: prev block idx (nullable) -> next block idx (nullable) -> block data
*/

type FileOffset = u64;

pub struct ASS {
    file: std::fs::File,
}
impl ASS {
    fn write_u64(&mut self, index: u64) {
        self.file.write_all(&index.to_be_bytes()).unwrap();
    }
    fn read_u64(&mut self) -> u64 {
        let mut result = [0u8; 8];
        self.file.read_exact(&mut result).unwrap();
        u64::from_be_bytes(result)
    }
    fn alloc(&mut self, amount: u64) -> FileOffset {
        if amount == 0 {
            return 1;
        }
        let mut is_free = true;
        self.file.seek(SeekFrom::Start(24));
        loop {
            let _prev_block_pos = self.read_u64();
            let next_block_pos = self.read_u64();

            if is_free {
                let cur_data_pos = self.file.seek(SeekFrom::Current(0)).unwrap();
                let cur_size = next_block_pos - cur_data_pos;
                if cur_size >= amount + 16 + 16 {
                    let cur_block_pos = cur_data_pos - 16;
                    self.write_u64(cur_block_pos);
                    self.write_u64(cur_block_pos + 16 + amount);
                }
            }

            self.file.seek(SeekFrom::Start(next_block_pos));
            is_free = !is_free;
        }
    }
    fn dealloc(&mut self, offset: u64) {
        if offset == 1 {
            return;
        }
        // ..rest of code..
    }
    pub fn get(&mut self, key: &[u8]) -> Vec<u8> {
    }
    pub fn set(&mut self, key: &[u8], value: &[u8]) {
    }
    pub fn open(path: impl AsRef<std::path::Path>) -> Self {
        let exists = std::fs::exists(&path).unwrap();
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&path)
            .unwrap();
        let mut this = Self { file };
        if !exists {
            this.write_u64(0);
            this.write_u64(0);
            this.write_u64(0);
            this.write_u64(0);
            this.write_u64(0);
        }
        this
    }
}
