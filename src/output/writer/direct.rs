use super::OutputWriter;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::PathBuf;
use tracing::info;

const DIRECT_ALIGN: usize = 4096;
const DIRECT_BUF_CAP: usize = 256 * 1024; // 256 KB write buffer

pub struct DirectWriter {
    file: File,
    buf: AlignedBuf,
    date: String,
    path: PathBuf,
}

impl DirectWriter {
    pub fn open(dir: &str, prefix: &str, date: &str) -> io::Result<Self> {
        fs::create_dir_all(dir)?;
        let path: PathBuf = [dir, &format!("{prefix}-{date}.jsonl")].iter().collect();
        info!(?path, mode = "direct", "Opening output file");

        let file = open_direct(&path)?;
        let buf = AlignedBuf::new(DIRECT_BUF_CAP, DIRECT_ALIGN);

        Ok(Self { file, buf, date: date.to_string(), path })
    }

    /// Flush aligned portion of the buffer. Remainder stays for next write.
    fn flush_aligned(&mut self) -> io::Result<()> {
        if self.buf.len == 0 {
            return Ok(());
        }

        let aligned_len = self.buf.len & !(DIRECT_ALIGN - 1);
        if aligned_len > 0 {
            self.file.write_all(&self.buf.data[..aligned_len])?;
            self.buf.data.copy_within(aligned_len..self.buf.len, 0);
            self.buf.len -= aligned_len;
        }
        Ok(())
    }

    /// Flush everything, padding to alignment if needed for O_DIRECT.
    fn flush_all(&mut self) -> io::Result<()> {
        if self.buf.len == 0 {
            return Ok(());
        }
        let padded_len = (self.buf.len + DIRECT_ALIGN - 1) & !(DIRECT_ALIGN - 1);
        self.buf.data[self.buf.len..padded_len].fill(0);
        self.file.write_all(&self.buf.data[..padded_len])?;

        let actual_pos = self.file.metadata()?.len();
        let over = (padded_len - self.buf.len) as u64;
        if over > 0 && actual_pos >= over {
            self.file.set_len(actual_pos - over)?;
        }
        self.buf.len = 0;
        Ok(())
    }
}

impl OutputWriter for DirectWriter {
    fn write_record(&mut self, data: &[u8]) -> io::Result<usize> {
        let total = data.len() + 1;

        if self.buf.len + total > self.buf.data.len() {
            self.flush_aligned()?;
        }
        if total > self.buf.data.len() {
            let new_cap = (total + DIRECT_ALIGN - 1) & !(DIRECT_ALIGN - 1);
            self.buf.data.resize(new_cap, 0);
        }

        self.buf.data[self.buf.len..self.buf.len + data.len()].copy_from_slice(data);
        self.buf.data[self.buf.len + data.len()] = b'\n';
        self.buf.len += total;

        if self.buf.len >= self.buf.data.len() - DIRECT_ALIGN {
            self.flush_aligned()?;
        }

        Ok(total)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.flush_aligned()?;
        self.file.flush()
    }

    fn close(mut self: Box<Self>) -> io::Result<()> {
        self.flush_all()?;
        self.file.flush()?;
        info!(path = ?self.path, "Closed direct I/O output file");
        Ok(())
    }

    fn date(&self) -> &str {
        &self.date
    }
}

// ──────────────────── Aligned Buffer ──────────────────────────

struct AlignedBuf {
    data: Vec<u8>,
    len: usize,
}

impl AlignedBuf {
    fn new(capacity: usize, align: usize) -> Self {
        use std::alloc::{alloc_zeroed, Layout};
        let layout = Layout::from_size_align(capacity, align).expect("Invalid layout");
        // SAFETY: layout has non-zero size; we zero-initialize and immediately
        // wrap in a Vec that owns the allocation.
        let data = unsafe {
            let ptr = alloc_zeroed(layout);
            assert!(!ptr.is_null(), "Allocation failed");
            Vec::from_raw_parts(ptr, capacity, capacity)
        };
        Self { data, len: 0 }
    }
}

// ──────────────────── Platform helpers ─────────────────────────

#[cfg(target_os = "macos")]
fn open_direct(path: &PathBuf) -> io::Result<File> {
    use std::os::fd::AsFd;
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    rustix::fs::fcntl_nocache(file.as_fd(), true)?;
    Ok(file)
}

#[cfg(target_os = "linux")]
fn open_direct(path: &PathBuf) -> io::Result<File> {
    use std::os::unix::fs::OpenOptionsExt;
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .custom_flags(rustix::fs::OFlags::DIRECT.bits() as i32)
        .open(path)?;
    Ok(file)
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn open_direct(path: &PathBuf) -> io::Result<File> {
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    Ok(file)
}
