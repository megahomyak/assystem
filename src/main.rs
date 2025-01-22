/// A sort of "memory allocator" for files
struct Fallocator {

}
const FILE_FORMAT_NAME: [char; 7] = ['A', 'S', 'S', ' ', 'v', '1', '\0'];
type FileIndex = u64;
/*
Database structure:
all: [format name (length added to every index)] [fixed memory (like stack)] [dynamic memory (like heap)]

Dynamic memory structure:
block: [prev block index] [block length] [flags: whether it's a free block and whether it's a last block]
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
