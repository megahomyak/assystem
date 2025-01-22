/// Sort of a "dynamic memory allocator" for files
struct Fallocator {
    heap_index: u64,
}
const FILE_FORMAT_NAME: [char; 7] = ['A', 'S', 'S', ' ', 'v', '1', '\0'];
type FileIndex = u64;
/*
Memory structure:
* File format name (['A', 'S', 'S', ' ', 'v', '1', '\0'])
* Root node 0 index (may be null)
* Root node 1 index (may be null)
* Root node content (may be null)
* First block flags
* After first block index (non-null)
* Heap (with blocks: [block flags, after block index (non-null), prev block index (non-null)])
*/
impl Fallocator {
    fn allocate(&mut self, bytes_amount: u64) -> FileIndex {

    }
    fn deallocate(&mut self, area: FileIndex) {
    
    }
}

pub struct ASS {

}
pub enum OpeningError {
    /// Not an ASS file of the needed version, unfortunately.
    Assless(),
    IO(std::io::Error),
}
impl ASS {
    pub fn open(path: impl AsRef<std::path::Path>) -> Result<Self, std::io::Error> {
        // =()=
        if path.as_ref() {

        }
    }
}

fn main() {
    println!("Hello, world!");
}
