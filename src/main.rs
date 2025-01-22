/// A sort of "memory allocator" for files
struct Fallocator {

}
const FILE_FORMAT_NAME: [char; 7] = ['A', 'S', 'S', ' ', 'v', '1', '\0'];
type FileIndex = u64;
/*
Database structure:
all: [format name (length added to every index)] [fixed memory (like stack)] [dynamic memory (like heap)]
fixed memory: [pair (for root)]
pair: [0 branch index (nullable)] [1 branch index (nullable)]
all: [format name (length added to every index)] [*first block*] [.. *rest of blocks* ..]
first block: [whether or not there's a block after it] [root 0 branch index (nullable)] [root 1 branch index (nullable)] [value index (nullable)] <- this block has index "0", btw! And must always be present, and is unique, which is why it's fine to use an index "0" for special cases
rest of blocks: [whether or not this block is taken] [prev block index (non-null)] [next block index (nullable)] [..any data..]
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
