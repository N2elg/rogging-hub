use super::OutputWriter;
use std::fs::{self, File, OpenOptions};
use std::io;
use std::path::PathBuf;
use tracing::{debug, info};

pub struct MmapWriter {
    file: File,
    mmap: memmap2::MmapMut,
    offset: usize,
    file_size: u64,
    chunk_size: usize,
    path: PathBuf,
    date: String,
}

impl MmapWriter {
    pub fn open(dir: &str, prefix: &str, date: &str, chunk_size: usize) -> io::Result<Self> {
        fs::create_dir_all(dir)?;
        let path: PathBuf = [dir, &format!("{prefix}-{date}.jsonl")].iter().collect();
        info!(?path, mode = "mmap", chunk_size, "Opening output file");

        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(&path)?;

        let existing_len = file.metadata()?.len();
        let offset = existing_len as usize;

        // Pre-allocate at least one chunk beyond current content.
        let file_size = if existing_len < chunk_size as u64 {
            chunk_size as u64
        } else {
            ((existing_len / chunk_size as u64) + 1) * chunk_size as u64
        };
        file.set_len(file_size)?;

        // SAFETY: single writer to this file region; we control the lifetime.
        let mmap = unsafe { memmap2::MmapMut::map_mut(&file)? };

        debug!(offset, file_size, "Mmap region mapped");
        Ok(Self { file, mmap, offset, file_size, chunk_size, path, date: date.to_string() })
    }

    fn grow(&mut self, needed: usize) -> io::Result<()> {
        // Flush current dirty pages before remapping.
        self.mmap.flush_async()?;

        let additional = std::cmp::max(self.chunk_size, needed);
        self.file_size += additional as u64;
        self.file.set_len(self.file_size)?;

        // SAFETY: same guarantees as open().
        self.mmap = unsafe { memmap2::MmapMut::map_mut(&self.file)? };
        debug!(new_size = self.file_size, "Extended mmap region");
        Ok(())
    }
}

impl OutputWriter for MmapWriter {
    fn write_record(&mut self, data: &[u8]) -> io::Result<usize> {
        let total = data.len() + 1; // +1 for newline

        if self.offset + total > self.mmap.len() {
            self.grow(total)?;
        }

        // memcpy into mapped region — no syscall.
        self.mmap[self.offset..self.offset + data.len()].copy_from_slice(data);
        self.mmap[self.offset + data.len()] = b'\n';
        self.offset += total;
        Ok(total)
    }

    fn flush(&mut self) -> io::Result<()> {
        if self.offset > 0 {
            self.mmap.flush_async()?;
        }
        Ok(())
    }

    fn close(self: Box<Self>) -> io::Result<()> {
        // Synchronous msync to guarantee data is on disk.
        self.mmap.flush()?;
        // Drop mapping before truncating.
        let offset = self.offset;
        let file = self.file;
        let path = self.path;
        drop(self.mmap);
        // Truncate pre-allocated file to actual written size.
        file.set_len(offset as u64)?;
        info!(?path, bytes = offset, "Closed mmap output file (truncated to actual size)");
        Ok(())
    }

    fn date(&self) -> &str {
        &self.date
    }
}
