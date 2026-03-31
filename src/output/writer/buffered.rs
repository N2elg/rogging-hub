use super::OutputWriter;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::PathBuf;
use tracing::info;

pub struct BufferedWriter {
    writer: io::BufWriter<File>,
    date: String,
    path: PathBuf,
}

impl BufferedWriter {
    pub fn open(dir: &str, prefix: &str, date: &str) -> io::Result<Self> {
        fs::create_dir_all(dir)?;
        let path: PathBuf = [dir, &format!("{prefix}-{date}.jsonl")].iter().collect();
        info!(?path, mode = "buffered", "Opening output file");

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;

        let writer = io::BufWriter::with_capacity(64 * 1024, file);
        Ok(Self { writer, date: date.to_string(), path })
    }
}

impl OutputWriter for BufferedWriter {
    fn write_record(&mut self, data: &[u8]) -> io::Result<usize> {
        self.writer.write_all(data)?;
        self.writer.write_all(b"\n")?;
        Ok(data.len() + 1)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }

    fn close(mut self: Box<Self>) -> io::Result<()> {
        self.writer.flush()?;
        info!(path = ?self.path, "Closed buffered output file");
        Ok(())
    }

    fn date(&self) -> &str {
        &self.date
    }
}
