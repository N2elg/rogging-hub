mod buffered;
mod direct;
mod mmap;

pub use buffered::BufferedWriter;
pub use direct::DirectWriter;
pub use mmap::MmapWriter;

use std::io;

// ──────────────────────────── Trait ────────────────────────────

pub trait OutputWriter: Send {
    fn write_record(&mut self, data: &[u8]) -> io::Result<usize>;
    fn flush(&mut self) -> io::Result<()>;
    fn close(self: Box<Self>) -> io::Result<()>;
    fn date(&self) -> &str;
}

// ──────────────────────── Factory ──────────────────────────────

pub fn create_writer(
    mode: &str,
    dir: &str,
    prefix: &str,
    date: &str,
    mmap_chunk_size: usize,
) -> io::Result<Box<dyn OutputWriter>> {
    match mode {
        "mmap" => Ok(Box::new(MmapWriter::open(dir, prefix, date, mmap_chunk_size)?)),
        "direct" => Ok(Box::new(DirectWriter::open(dir, prefix, date)?)),
        _ => Ok(Box::new(BufferedWriter::open(dir, prefix, date)?)),
    }
}
