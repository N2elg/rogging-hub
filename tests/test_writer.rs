use std::fs::File;
use std::io::Read;

use RoggingHub::output::writer::{self, OutputWriter};

fn temp_dir() -> tempfile::TempDir {
    tempfile::tempdir().unwrap()
}

// ──────────────── Buffered ────────────────

mod buffered {
    use super::*;
    use RoggingHub::output::writer::BufferedWriter;

    #[test]
    fn write_and_read_back() {
        let dir = temp_dir();
        let mut w = BufferedWriter::open(dir.path().to_str().unwrap(), "test", "2026-04-01").unwrap();
        w.write_record(br#"{"a":1}"#).unwrap();
        w.write_record(br#"{"b":2}"#).unwrap();
        let boxed: Box<dyn OutputWriter> = Box::new(w);
        boxed.close().unwrap();

        let path = dir.path().join("test-2026-04-01.jsonl");
        let mut content = String::new();
        File::open(&path).unwrap().read_to_string(&mut content).unwrap();
        assert_eq!(content, "{\"a\":1}\n{\"b\":2}\n");
    }

    #[test]
    fn write_record_returns_correct_size() {
        let dir = temp_dir();
        let mut w = BufferedWriter::open(dir.path().to_str().unwrap(), "test", "2026-04-01").unwrap();
        let n = w.write_record(b"hello").unwrap();
        assert_eq!(n, 6); // "hello" + "\n"
        Box::new(w).close().unwrap();
    }

    #[test]
    fn date_returns_creation_date() {
        let dir = temp_dir();
        let w = BufferedWriter::open(dir.path().to_str().unwrap(), "test", "2026-04-01").unwrap();
        assert_eq!(w.date(), "2026-04-01");
        Box::new(w).close().unwrap();
    }

    #[test]
    fn append_to_existing_file() {
        let dir = temp_dir();
        {
            let mut w = BufferedWriter::open(dir.path().to_str().unwrap(), "test", "2026-04-01").unwrap();
            w.write_record(b"first").unwrap();
            Box::new(w).close().unwrap();
        }
        {
            let mut w = BufferedWriter::open(dir.path().to_str().unwrap(), "test", "2026-04-01").unwrap();
            w.write_record(b"second").unwrap();
            Box::new(w).close().unwrap();
        }
        let path = dir.path().join("test-2026-04-01.jsonl");
        let mut content = String::new();
        File::open(&path).unwrap().read_to_string(&mut content).unwrap();
        assert_eq!(content, "first\nsecond\n");
    }

    #[test]
    fn flush_does_not_error() {
        let dir = temp_dir();
        let mut w = BufferedWriter::open(dir.path().to_str().unwrap(), "test", "2026-04-01").unwrap();
        w.write_record(b"data").unwrap();
        w.flush().unwrap();
        Box::new(w).close().unwrap();
    }
}

// ──────────────── Mmap ────────────────

mod mmap {
    use super::*;
    use RoggingHub::output::writer::MmapWriter;

    const SMALL_CHUNK: usize = 4096;

    #[test]
    fn write_and_close_truncates() {
        let dir = temp_dir();
        let mut w = MmapWriter::open(dir.path().to_str().unwrap(), "test", "2026-04-01", SMALL_CHUNK).unwrap();
        w.write_record(br#"{"a":1}"#).unwrap();
        w.write_record(br#"{"b":2}"#).unwrap();
        let boxed: Box<dyn OutputWriter> = Box::new(w);
        boxed.close().unwrap();

        let path = dir.path().join("test-2026-04-01.jsonl");
        let meta = std::fs::metadata(&path).unwrap();
        let expected = b"{\"a\":1}\n{\"b\":2}\n";
        assert_eq!(meta.len(), expected.len() as u64);

        let mut content = String::new();
        File::open(&path).unwrap().read_to_string(&mut content).unwrap();
        assert_eq!(content, "{\"a\":1}\n{\"b\":2}\n");
    }

    #[test]
    fn grow_beyond_initial_chunk() {
        let dir = temp_dir();
        let tiny_chunk = 64;
        let mut w = MmapWriter::open(dir.path().to_str().unwrap(), "test", "2026-04-01", tiny_chunk).unwrap();

        let record = b"abcdefghijklmnopqrstuvwxyz012345";
        for _ in 0..10 {
            w.write_record(record).unwrap();
        }
        Box::new(w).close().unwrap();

        let path = dir.path().join("test-2026-04-01.jsonl");
        let mut content = String::new();
        File::open(&path).unwrap().read_to_string(&mut content).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 10);
        for line in lines {
            assert_eq!(line, std::str::from_utf8(record).unwrap());
        }
    }

    #[test]
    fn date_returns_creation_date() {
        let dir = temp_dir();
        let w = MmapWriter::open(dir.path().to_str().unwrap(), "test", "2026-04-01", SMALL_CHUNK).unwrap();
        assert_eq!(w.date(), "2026-04-01");
        Box::new(w).close().unwrap();
    }

    #[test]
    fn flush_does_not_error() {
        let dir = temp_dir();
        let mut w = MmapWriter::open(dir.path().to_str().unwrap(), "test", "2026-04-01", SMALL_CHUNK).unwrap();
        w.write_record(b"data").unwrap();
        w.flush().unwrap();
        Box::new(w).close().unwrap();
    }
}

// ──────────────── Direct ────────────────

mod direct {
    use super::*;
    use RoggingHub::output::writer::DirectWriter;

    #[test]
    fn write_and_close() {
        let dir = temp_dir();
        let mut w = DirectWriter::open(dir.path().to_str().unwrap(), "test", "2026-04-01").unwrap();
        w.write_record(br#"{"a":1}"#).unwrap();
        w.write_record(br#"{"b":2}"#).unwrap();
        let boxed: Box<dyn OutputWriter> = Box::new(w);
        boxed.close().unwrap();

        let path = dir.path().join("test-2026-04-01.jsonl");
        let mut content = String::new();
        File::open(&path).unwrap().read_to_string(&mut content).unwrap();
        assert_eq!(content, "{\"a\":1}\n{\"b\":2}\n");
    }

    #[test]
    fn write_record_returns_correct_size() {
        let dir = temp_dir();
        let mut w = DirectWriter::open(dir.path().to_str().unwrap(), "test", "2026-04-01").unwrap();
        let n = w.write_record(b"hello").unwrap();
        assert_eq!(n, 6);
        Box::new(w).close().unwrap();
    }

    #[test]
    fn large_records_beyond_buffer() {
        let dir = temp_dir();
        let mut w = DirectWriter::open(dir.path().to_str().unwrap(), "test", "2026-04-01").unwrap();
        let big = vec![b'x'; 300_000];
        w.write_record(&big).unwrap();
        Box::new(w).close().unwrap();

        let path = dir.path().join("test-2026-04-01.jsonl");
        let mut content = Vec::new();
        File::open(&path).unwrap().read_to_end(&mut content).unwrap();
        assert_eq!(content.len(), 300_001);
        assert_eq!(content.last(), Some(&b'\n'));
    }

    #[test]
    fn many_small_records() {
        let dir = temp_dir();
        let mut w = DirectWriter::open(dir.path().to_str().unwrap(), "test", "2026-04-01").unwrap();
        for i in 0..1000 {
            let record = format!("{{\"i\":{i}}}");
            w.write_record(record.as_bytes()).unwrap();
        }
        Box::new(w).close().unwrap();

        let path = dir.path().join("test-2026-04-01.jsonl");
        let mut content = String::new();
        File::open(&path).unwrap().read_to_string(&mut content).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 1000);
        assert_eq!(lines[0], "{\"i\":0}");
        assert_eq!(lines[999], "{\"i\":999}");
    }

    #[test]
    fn date_returns_creation_date() {
        let dir = temp_dir();
        let w = DirectWriter::open(dir.path().to_str().unwrap(), "test", "2026-04-01").unwrap();
        assert_eq!(w.date(), "2026-04-01");
        Box::new(w).close().unwrap();
    }
}

// ──────────────── Factory ────────────────

mod factory {
    use super::*;

    #[test]
    fn factory_creates_buffered_by_default() {
        let dir = temp_dir();
        let mut w = writer::create_writer("buffered", dir.path().to_str().unwrap(), "t", "2026-04-01", 4096).unwrap();
        w.write_record(b"test").unwrap();
        assert_eq!(w.date(), "2026-04-01");
        w.close().unwrap();
    }

    #[test]
    fn factory_creates_mmap() {
        let dir = temp_dir();
        let mut w = writer::create_writer("mmap", dir.path().to_str().unwrap(), "t", "2026-04-01", 4096).unwrap();
        w.write_record(b"test").unwrap();
        w.close().unwrap();
    }

    #[test]
    fn factory_creates_direct() {
        let dir = temp_dir();
        let mut w = writer::create_writer("direct", dir.path().to_str().unwrap(), "t", "2026-04-01", 4096).unwrap();
        w.write_record(b"test").unwrap();
        w.close().unwrap();
    }

    #[test]
    fn unknown_mode_falls_back_to_buffered() {
        let dir = temp_dir();
        let mut w = writer::create_writer("unknown", dir.path().to_str().unwrap(), "t", "2026-04-01", 4096).unwrap();
        w.write_record(b"test").unwrap();
        w.close().unwrap();
    }
}
