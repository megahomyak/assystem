use std::{
    io::{Read, Seek, SeekFrom, Write},
    rc::Rc,
};
/*
Memory structure: file header, root node, blocks

"File header" is just the name of the file format

Node structure: false branch data pos (nul?), true branch data pos (nul?), content block pos (nul?)

Block structure: prev block pos (nul? only in first block), block length, next block pos (nul?)

The first block must be present and empty
*/

const FILE_HEADER: [u8; 7] = *b"ASS v1\0";
mod offsets {
    pub const ROOT_NODE: u64 = super::FILE_HEADER.len() as u64;
    pub const BLOCKS: u64 = ROOT_NODE + 24;
}

type DataPosition = u64;
type BlockPosition = u64;

fn bits<'a>(bytes: &'a [u8]) -> impl Iterator<Item = bool> + 'a {
    struct BitIter<'a> {
        bytes: std::slice::Iter<'a, u8>,
        mask: u8,
        curbyte: u8,
    }
    impl<'a> Iterator for BitIter<'a> {
        type Item = bool;

        fn next(&mut self) -> Option<Self::Item> {
            if self.mask == 0b1000_0000 {
                self.curbyte = *self.bytes.next()?;
            }
            let result = self.mask & self.curbyte != 0;
            self.mask = self.mask.rotate_right(1);
            Some(result)
        }
    }
    BitIter {
        bytes: bytes.iter(),
        mask: 0b1000_0000,
        curbyte: 0,
    }
}

struct PrevPlan {
    ref_: Rc<Plan>,
    true_branch: bool,
}
struct Plan {
    prev: Option<PrevPlan>,
    pos: u64,
}
impl Plan {
    fn gather_key(&self) -> Vec<u8> {
        let mut result = Vec::new();

        let mut curbyte: u8 = 0;
        let mut mask: u8 = 0b0000_0001;

        let mut cur_prev = &self.prev;

        while let Some(prev) = cur_prev {
            if prev.true_branch {
                curbyte |= mask;
            }
            if mask == 0b1000_0000 {
                result.push(curbyte);
                curbyte = 0;
            }
            mask = mask.rotate_left(1);
            cur_prev = &prev.ref_.prev;
        }

        result.reverse();
        result
    }
}

pub struct Lister<'a, F> {
    ass: &'a mut ASS<F>,
    plans: Vec<Plan>,
}
impl<'a, F: ASSFile> Iterator for Lister<'a, F> {
    type Item = (Vec<u8>, Vec<u8>);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let plan = self.plans.pop()?;
            self.ass.file.seek(SeekFrom::Start(plan.pos)).unwrap();
            let plan = Rc::new(plan);
            for true_branch in [false, true] {
                let branch_data_pos = self.ass.read_u64();
                if branch_data_pos != DATA_DOES_NOT_EXIST_POS {
                    self.plans.push(Plan {
                        pos: branch_data_pos,
                        prev: Some(PrevPlan {
                            ref_: plan.clone(),
                            true_branch,
                        }),
                    });
                }
            }
            let content_block_pos = self.ass.read_u64();
            if content_block_pos != DATA_DOES_NOT_EXIST_POS {
                let key = plan.gather_key();
                let value = self.ass.read_block(content_block_pos);
                return Some((key, value));
            }
        }
    }
}

pub trait ASSFile: Write + Read + Seek {
    /// Truncates the file at its current position
    fn truncate(&mut self) -> std::io::Result<()>;
}
impl ASSFile for std::io::Cursor<Vec<u8>> {
    fn truncate(&mut self) -> std::io::Result<()> {
        let curpos = self.seek(SeekFrom::Current(0)).unwrap();
        self.get_mut().truncate(curpos.try_into().unwrap());
        Ok(())
    }
}
impl ASSFile for std::fs::File {
    fn truncate(&mut self) -> std::io::Result<()> {
        let curpos = self.seek(SeekFrom::Current(0)).unwrap();
        self.set_len(curpos)
    }
}

const EMPTY_VALUE_BLOCK_POS: u64 = 1;
const DATA_DOES_NOT_EXIST_POS: u64 = 0;

pub struct ASS<F> {
    file: F,
}
impl<F: ASSFile> ASS<F> {
    fn write_u64(&mut self, index: u64) {
        self.file.write_all(&index.to_be_bytes()).unwrap();
    }
    fn read_u64(&mut self) -> u64 {
        let mut result = [0u8; 8];
        self.file.read_exact(&mut result).unwrap();
        u64::from_be_bytes(result)
    }
    fn alloc(&mut self, data: &[u8]) -> BlockPosition {
        if data.len() == 0 {
            return EMPTY_VALUE_BLOCK_POS;
        }
        let data_len: u64 = data.len().try_into().unwrap();
        self.file.seek(SeekFrom::Start(offsets::BLOCKS)).unwrap();
        loop {
            let _prev_block_pos = self.read_u64();
            let block_length = self.read_u64();
            let next_block_pos = self.read_u64();

            if next_block_pos == DATA_DOES_NOT_EXIST_POS {
                let data_pos = self.tell();
                self.file
                    .seek(SeekFrom::Current(block_length.try_into().unwrap()))
                    .unwrap();
                self.write_u64(data_pos - 24);
                self.write_u64(data_len);
                self.write_u64(DATA_DOES_NOT_EXIST_POS);
                self.file.write_all(&data).unwrap();
                self.file.seek(SeekFrom::Start(data_pos - 8)).unwrap();
                let new_block_pos = data_pos + block_length;
                self.write_u64(new_block_pos);
                return new_block_pos;
            } else {
                let data_pos = self.tell();
                let free_space_length = (next_block_pos - data_pos) - block_length;
                if free_space_length >= data_len + 24 {
                    self.file
                        .seek(SeekFrom::Current(block_length.try_into().unwrap()))
                        .unwrap();
                    self.write_u64(data_pos - 24);
                    self.write_u64(data_len);
                    self.write_u64(next_block_pos);
                    self.file.write_all(&data).unwrap();
                    self.file.seek(SeekFrom::Start(data_pos - 8)).unwrap();
                    let new_block_pos = data_pos + block_length;
                    self.write_u64(new_block_pos);
                    if next_block_pos != DATA_DOES_NOT_EXIST_POS {
                        self.file.seek(SeekFrom::Start(next_block_pos)).unwrap();
                        self.write_u64(new_block_pos);
                    }
                    return new_block_pos;
                }
            }

            self.file.seek(SeekFrom::Start(next_block_pos)).unwrap();
        }
    }
    fn dealloc(&mut self, pos: BlockPosition) {
        if pos == EMPTY_VALUE_BLOCK_POS {
            return;
        }
        self.file.seek(SeekFrom::Start(pos)).unwrap();
        let prev_block_pos = self.read_u64();
        let _block_length = self.read_u64();
        let next_block_pos = self.read_u64();
        if next_block_pos == DATA_DOES_NOT_EXIST_POS {
            self.file.seek(SeekFrom::Start(prev_block_pos + 8)).unwrap();
            let prev_block_len = self.read_u64();
            self.file
                .seek(SeekFrom::Current(
                    i64::try_from(prev_block_len).unwrap() + 8,
                ))
                .unwrap();
            self.file.truncate().unwrap();
        } else {
            self.file.seek(SeekFrom::Start(next_block_pos)).unwrap();
            self.write_u64(prev_block_pos);
        }
        self.file
            .seek(SeekFrom::Start(prev_block_pos + 16))
            .unwrap();
        self.write_u64(next_block_pos);
    }
    fn read_block(&mut self, pos: BlockPosition) -> Vec<u8> {
        if pos == EMPTY_VALUE_BLOCK_POS {
            return Vec::new();
        }
        self.file.seek(SeekFrom::Start(pos)).unwrap();
        let _prev_block_pos = self.read_u64();
        let block_length = self.read_u64();
        let _next_block_pos = self.read_u64();
        let mut result = vec![0u8; block_length.try_into().unwrap()];
        self.file.read_exact(&mut result).unwrap();
        result
    }
    fn tell(&mut self) -> u64 {
        self.file.seek(SeekFrom::Current(0)).unwrap()
    }
    pub fn get(&mut self, key: &[u8]) -> Option<Vec<u8>> {
        self.file.seek(SeekFrom::Start(offsets::ROOT_NODE)).unwrap();
        for bit in bits(key) {
            if bit {
                self.file.seek(SeekFrom::Current(8)).unwrap();
            }
            let branch_data_position = self.read_u64();
            if branch_data_position == DATA_DOES_NOT_EXIST_POS {
                return None;
            }
            self.file
                .seek(SeekFrom::Start(branch_data_position))
                .unwrap();
        }
        self.file.seek(SeekFrom::Current(16)).unwrap();
        let content_block_pos = self.read_u64();
        if content_block_pos == DATA_DOES_NOT_EXIST_POS {
            None
        } else {
            Some(self.read_block(content_block_pos))
        }
    }
    pub fn set(&mut self, key: &[u8], value: &[u8]) -> Option<Vec<u8>> {
        self.file.seek(SeekFrom::Start(offsets::ROOT_NODE)).unwrap();
        for bit in bits(key) {
            if bit {
                self.file.seek(SeekFrom::Current(8)).unwrap();
            }
            let branch_data_pos_pos = self.tell();
            let mut branch_data_pos = self.read_u64();
            if branch_data_pos == DATA_DOES_NOT_EXIST_POS {
                let new_node_data_pos = self.alloc(&[0u8; 24]) + 24;
                self.file
                    .seek(SeekFrom::Start(branch_data_pos_pos))
                    .unwrap();
                self.write_u64(new_node_data_pos);
                branch_data_pos = new_node_data_pos;
            }
            self.file.seek(SeekFrom::Start(branch_data_pos)).unwrap();
        }
        self.file.seek(SeekFrom::Current(16)).unwrap();
        let content_block_pos_pos = self.tell();
        let old_content_block_pos = self.read_u64();
        let previous_value = if old_content_block_pos == DATA_DOES_NOT_EXIST_POS {
            None
        } else {
            let previous_value = self.read_block(old_content_block_pos);
            self.dealloc(old_content_block_pos);
            Some(previous_value)
        };
        let new_content_block_pos = self.alloc(value);
        self.file
            .seek(SeekFrom::Start(content_block_pos_pos))
            .unwrap();
        self.write_u64(new_content_block_pos);
        previous_value
    }
    pub fn remove(&mut self, key: &[u8]) -> Option<Vec<u8>> {
        struct Decision {
            pos: DataPosition,
            true_branch: bool,
        }
        let mut decisions = Vec::new();
        let mut cur_data_pos: DataPosition = offsets::ROOT_NODE;
        self.file.seek(SeekFrom::Start(cur_data_pos)).unwrap();
        for bit in bits(key) {
            if bit {
                self.file.seek(SeekFrom::Current(8)).unwrap();
            }
            let branch_data_position = self.read_u64();
            if branch_data_position == DATA_DOES_NOT_EXIST_POS {
                return None;
            }
            self.file
                .seek(SeekFrom::Start(branch_data_position))
                .unwrap();
            decisions.push(Decision {
                pos: cur_data_pos,
                true_branch: bit,
            });
            cur_data_pos = branch_data_position;
        }
        let node_pos = self.tell();
        self.file.seek(SeekFrom::Current(16)).unwrap();
        let content_block_pos = self.read_u64();
        let previous_value = if content_block_pos == DATA_DOES_NOT_EXIST_POS {
            None
        } else {
            let previous_value = self.read_block(content_block_pos);
            self.dealloc(content_block_pos);
            Some(previous_value)
        };
        self.file.seek(SeekFrom::Start(node_pos + 16)).unwrap();
        self.write_u64(DATA_DOES_NOT_EXIST_POS);
        let mut cur_data_pos = node_pos;
        while let Some(decision) = decisions.pop() {
            self.file.seek(SeekFrom::Start(cur_data_pos)).unwrap();
            let false_branch_data_pos = self.read_u64();
            let true_branch_data_pos = self.read_u64();
            let content_block_pos = self.read_u64();
            if false_branch_data_pos == DATA_DOES_NOT_EXIST_POS
                && true_branch_data_pos == DATA_DOES_NOT_EXIST_POS
                && content_block_pos == DATA_DOES_NOT_EXIST_POS
            {
                self.dealloc(cur_data_pos - 24);
                self.file.seek(SeekFrom::Start(decision.pos)).unwrap();
                if decision.true_branch {
                    self.file.seek(SeekFrom::Current(8)).unwrap();
                }
                self.write_u64(DATA_DOES_NOT_EXIST_POS);
                cur_data_pos = decision.pos;
            } else {
                break;
            }
        }
        previous_value
    }
    pub fn list(&mut self) -> Lister<F> {
        Lister {
            ass: self,
            plans: vec![Plan {
                pos: offsets::ROOT_NODE,
                prev: None,
            }],
        }
    }
    fn open_any(file: F, exists: bool) -> Result<Self, OpeningError> {
        let mut ass = Self { file };
        if exists {
            let mut header_buf = [0u8; FILE_HEADER.len()];
            ass.file
                .read_exact(&mut header_buf)
                .map_err(|_| OpeningError::Assless())?;
            if header_buf != FILE_HEADER {
                return Err(OpeningError::Assless());
            }
        } else {
            ass.file.write_all(&FILE_HEADER).unwrap();
            ass.write_u64(DATA_DOES_NOT_EXIST_POS);
            ass.write_u64(DATA_DOES_NOT_EXIST_POS);
            ass.write_u64(DATA_DOES_NOT_EXIST_POS);
            ass.write_u64(DATA_DOES_NOT_EXIST_POS);
            ass.write_u64(0);
            ass.write_u64(DATA_DOES_NOT_EXIST_POS);
        }
        Ok(ass)
    }
}

#[derive(Debug)]
pub enum OpeningError {
    /// The file by the given path is not an ASS file of the needed version
    Assless(),
    IO(std::io::Error),
}

impl ASS<std::fs::File> {
    pub fn open(path: impl AsRef<std::path::Path>) -> Result<Self, OpeningError> {
        let exists = std::fs::exists(&path).map_err(|err| OpeningError::IO(err))?;
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)
            .map_err(|err| OpeningError::IO(err))?;
        Self::open_any(file, exists)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set_get() -> ASS<impl ASSFile> {
        let mut ass = ASS::open_any(std::io::Cursor::new(Vec::new()), false).unwrap();
        assert_eq!(ass.set(b"Drunk", b"Driving"), None);
        assert_eq!(ass.set(b"Spongebob", b"Squarewave"), None);
        assert_eq!(ass.set(b"Drunk", b"Driving"), Some(v(b"Driving")));
        assert_eq!(ass.get(b"Spongebob"), Some(v(b"Squarewave")));
        assert_eq!(ass.get(b"Drunk"), Some(v(b"Driving")));
        assert_eq!(ass.get(b"DISTONN"), None);
        ass
    }
    #[test]
    fn test_set_get() {
        set_get();
    }

    fn len<F: ASSFile>(ass: &mut ASS<F>) -> u64 {
        ass.file.seek(SeekFrom::End(0)).unwrap()
    }

    fn v(b: &[u8]) -> Vec<u8> {
        Vec::from(b)
    }

    #[test]
    fn test_replacing() {
        let mut ass = set_get();

        let len_1 = len(&mut ass);

        assert_eq!(
            ass.set(b"Spongebob", b"Squarepants"),
            Some(Vec::from(b"Squarewave"))
        );

        let len_2 = len(&mut ass);

        assert_eq!(len_1, len_2 - 1);

        assert_eq!(
            ass.set(b"Spongebob", b"Squarepants"),
            Some(Vec::from(b"Squarepants"))
        );

        let len_3 = len(&mut ass);

        assert_eq!(len_2, len_3);
    }

    #[test]
    fn test_listing() {
        let mut ass = set_get();

        assert_eq!(
            ass.list().collect::<Vec<_>>(),
            vec![
                (v(b"Spongebob"), v(b"Squarewave")),
                (v(b"Drunk"), v(b"Driving"))
            ]
        );
    }

    #[test]
    fn test_removing() {
        let mut ass = set_get();

        assert_eq!(ass.remove(b"Spongebob"), Some(v(b"Squarewave")));
        assert_eq!(ass.remove(b"Spongebob"), None);
    }

    #[test]
    fn test_branch_reduction() {
        let mut ass = set_get();

        let source_len = len(&mut ass);

        assert_eq!(ass.set(b"Spongebob1", b"TEST"), None);

        let len_after_addition = len(&mut ass);

        assert_eq!(source_len, len_after_addition - (24 * 2) * 8 - 24 - 4);

        assert_eq!(ass.remove(b"Spongebob1"), Some(v(b"TEST")));
        assert_eq!(ass.remove(b"Spongebob1"), None);
        assert_eq!(ass.get(b"Spongebob"), Some(v(b"Squarewave")));

        let len_after_removal = len(&mut ass);

        assert_eq!(source_len, len_after_removal);
    }
}
