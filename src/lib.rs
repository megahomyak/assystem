use std::{
    io::{Read, Seek, SeekFrom, Write},
    rc::Rc,
};
/*
Memory structure: root node, blocks

Node structure: 0 branch data pos (nul?), 1 branch data pos (nul?), content block pos (nul?)

Block structure: prev block pos (nul?), block length, next block pos (nul?)

The first block must be present and empty
*/

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
    fn gather(&self) -> Vec<u8> {
        let mut result = Vec::new();

        let mut curbyte: u8 = 0;
        let mut mask: u8 = 0b0000_0001;

        let mut cur_prev = &self.prev;

        while let Some(prev) = cur_prev {
            if prev.true_branch {
                curbyte |= mask;
            }
            if curbyte == 0b1000_0000 {
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
                if branch_data_pos != 0 {
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
            if content_block_pos != 0 {
                let key = plan.gather();
                let value = self.ass.read_block(content_block_pos);
                return Some((key, value));
            }
        }
    }
}

pub trait ASSFile: Write + Read + Seek {}
impl<T: Write + Read + Seek> ASSFile for T {}

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
            return 1;
        }
        let data_len: u64 = data.len().try_into().unwrap();
        self.file.seek(SeekFrom::Start(24)).unwrap();
        loop {
            let _prev_block_pos = self.read_u64();
            let block_length = self.read_u64();
            let next_block_pos = self.read_u64();

            if next_block_pos == 0 {
                let data_pos = self.tell();
                self.file
                    .seek(SeekFrom::Current(block_length.try_into().unwrap()))
                    .unwrap();
                self.write_u64(data_pos - 24);
                self.write_u64(data_len);
                self.write_u64(0);
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
                    let new_block_pos = data_pos + block_length;
                    self.file.seek(SeekFrom::Start(data_pos - 8)).unwrap();
                    self.write_u64(new_block_pos);
                    if next_block_pos != 0 {
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
        if pos == 1 {
            return;
        }
        self.file.seek(SeekFrom::Start(pos)).unwrap();
        let prev_block_pos = self.read_u64();
        let _block_length = self.read_u64();
        let next_block_pos = self.read_u64();
        self.file.seek(SeekFrom::Start(prev_block_pos)).unwrap();
        self.write_u64(next_block_pos);
    }
    fn read_block(&mut self, pos: BlockPosition) -> Vec<u8> {
        if pos == 1 {
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
        self.file.seek(SeekFrom::Start(0)).unwrap();
        for bit in bits(key) {
            if bit {
                self.file.seek(SeekFrom::Current(8)).unwrap();
            }
            let branch_data_position = self.read_u64();
            if branch_data_position == 0 {
                return None;
            }
            self.file
                .seek(SeekFrom::Start(branch_data_position))
                .unwrap();
        }
        self.file.seek(SeekFrom::Current(16)).unwrap();
        let content_block_pos = self.read_u64();
        if content_block_pos == 0 {
            None
        } else {
            Some(self.read_block(content_block_pos))
        }
    }
    pub fn set(&mut self, key: &[u8], value: &[u8]) -> Option<Vec<u8>> {
        self.file.seek(SeekFrom::Start(0)).unwrap();
        for bit in bits(key) {
            if bit {
                self.file.seek(SeekFrom::Current(8)).unwrap();
            }
            let branch_data_pos_pos = self.tell();
            let mut branch_data_pos = self.read_u64();
            if branch_data_pos == 0 {
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
        let previous_value = if old_content_block_pos == 0 {
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
        let mut cur_data_pos: DataPosition = 0;
        self.file.seek(SeekFrom::Start(cur_data_pos)).unwrap();
        for bit in bits(key) {
            if bit {
                self.file.seek(SeekFrom::Current(8)).unwrap();
            }
            let branch_data_position = self.read_u64();
            if branch_data_position == 0 {
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
        let node_pos_pos = self.tell();
        self.file.seek(SeekFrom::Current(16)).unwrap();
        let content_block_pos = self.read_u64();
        let previous_value = if content_block_pos == 0 {
            None
        } else {
            let previous_value = self.read_block(content_block_pos);
            self.dealloc(content_block_pos);
            Some(previous_value)
        };
        self.file.seek(SeekFrom::Start(node_pos_pos + 16)).unwrap();
        self.write_u64(0);
        let mut cur_data_pos = node_pos_pos;
        while let Some(decision) = decisions.pop() {
            self.file.seek(SeekFrom::Start(cur_data_pos)).unwrap();
            let false_branch_data_pos = self.read_u64();
            let true_branch_data_pos = self.read_u64();
            let content_block_pos = self.read_u64();
            if false_branch_data_pos == 0 && true_branch_data_pos == 0 && content_block_pos == 0 {
                self.dealloc(cur_data_pos - 24);
                self.file.seek(SeekFrom::Start(decision.pos)).unwrap();
                if decision.true_branch {
                    self.file.seek(SeekFrom::Current(8)).unwrap();
                }
                self.write_u64(0);
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
            plans: vec![Plan { pos: 0, prev: None }],
        }
    }
    fn init(&mut self) {
        self.write_u64(0);
        self.write_u64(0);
        self.write_u64(0);
        self.write_u64(0);
        self.write_u64(0);
        self.write_u64(0);
    }
    pub fn open(path: impl AsRef<std::path::Path>) -> ASS<std::fs::File> {
        let exists = std::fs::exists(&path).unwrap();
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&path)
            .unwrap();
        let mut this = ASS { file };
        if !exists {
            this.init();
        }
        this
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let file = std::io::Cursor::new(Vec::<u8>::new());
        fn len(ass: &mut ASS<std::io::Cursor<Vec<u8>>>) -> u64 {
            ass.file.seek(SeekFrom::End(0)).unwrap()
        }
        fn v(b: &[u8]) -> Vec<u8> {
            Vec::from(b)
        }
        let mut ass = ASS { file };
        ass.init();
        assert_eq!(ass.set(b"Spongebob", b"Squarewave"), None);
        assert_eq!(ass.set(b"Drunk", b"Driving"), None);
        assert_eq!(ass.get(b"Spongebob"), Some(v(b"Squarewave")));
        assert_eq!(ass.get(b"Drunk"), Some(v(b"Driving")));
        assert_eq!(ass.get(b"DISTONN"), None);
        let old_len = len(&mut ass);

        assert_eq!(ass.set(b"Spongebob", b"Squarepants"), Some(Vec::from(b"Squarewave")));

        let new_len = len(&mut ass);

        assert_eq!(old_len, new_len - 1);

        let items: Vec<_> = ass.list().collect();

        assert_eq!(items, vec![(v(b"Spongebob"), v(b"Squarepants")), (v(b"Drunk"), v(b"Driving"))])
    }
}
