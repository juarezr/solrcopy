use log::error;
use zip::result::ZipResult;
use zip::write::FileOptions;
use zip::ZipWriter;

use std::io::Write;
use std::path::{Path, PathBuf};

use crate::args::Backup;
use crate::fails::*;
use crate::helpers::*;
use crate::steps::*;

// TODO: split in multiple files of constant size
// TODO: limit file size based on zip.stats.bytes_written

// region Archiver Iterator

impl Backup {
    pub fn get_writer(&self, num_found: u64) -> Result<Archiver, BoxedError> {
        let (pattern, size) = self.get_archive_pattern(num_found);
        let res = Archiver::write_on(&self.into, pattern, size);
        Ok(res)
    }
}

pub struct DocumentArchiver<T> {
    it: T,
    archiver: Archiver,
}

impl<T> Iterator for DocumentArchiver<T>
where
    T: Iterator<Item = Documents>,
{
    type Item = Step;

    fn next(&mut self) -> Option<Step> {
        let next = self.it.next();

        if let Some(result) = next {
            let step = result.step;
            let filename = step.get_docs_filename();
            let docs = result.docs;
            let failed = self.archiver.write_file(&filename, &docs);
            if let Err(cause) = failed {
                error!("Error writing file {}: {}", filename, cause);
            } else {
                return Some(step);
            }
        }
        None
    }
}

pub trait DocumentIterator
where
    Self: Sized + Iterator<Item = Documents>,
{
    fn store_documents(self, archiver: Archiver) -> DocumentArchiver<Self>;
}

/// Wraps the previous iterator to process it's items.
impl<T: Iterator<Item = Documents>> DocumentIterator for T {
    fn store_documents(self, archiver: Archiver) -> DocumentArchiver<Self> {
        DocumentArchiver { it: self, archiver }
    }
}

// endregion

// region Archiver

type Compressor = ZipWriter<std::fs::File>;

pub struct Archiver {
    writer: Option<Compressor>,
    folder: PathBuf,
    file_pattern: String,
    pattern_size: usize,
    sequence: u64,
}

impl Archiver {
    fn write_on(dir: &PathBuf, pattern: String, size: usize) -> Self {
        Archiver {
            writer: None,
            folder: dir.to_owned(),
            file_pattern: pattern,
            pattern_size: size,
            sequence: 0,
        }
    }

    pub fn create_archive(&mut self) -> ZipResult<()> {
        self.close_archive()?;

        self.sequence += 1;
        let seq = format!("{}", self.sequence)
            .as_str()
            .pad_0(self.pattern_size);
        let file_name = self.file_pattern.replace("{}", &seq);
        let zip_path = Path::new(&self.folder);
        let zip_name = Path::new(&file_name);
        let zip_file = zip_path.join(&zip_name);

        let file = std::fs::File::create(&zip_file)?;
        let zip = zip::ZipWriter::new(file);

        self.writer = Some(zip);
        Ok(())
    }

    pub fn write_file(&mut self, filename: &str, docs: &str) -> ZipResult<()> {
        if self.writer.is_none() {
            self.create_archive()?;
        }
        let bytes = docs.as_bytes();

        let zip = self.writer.as_mut().unwrap();

        let opts: FileOptions = FileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(0o644);

        zip.start_file(filename, opts)?;
        zip.write_all(bytes)?;
        zip.flush()?;
        Ok(())
    }

    pub fn close_archive(&mut self) -> ZipResult<()> {
        if self.writer.is_some() {
            self.writer.as_mut().unwrap().finish()?;
        }
        self.writer = None;
        Ok(())
    }
}

impl Drop for Archiver {
    fn drop(&mut self) {
        let fail = self.close_archive();
        if let Err(cause) = fail {
            error!("> Dropping {}", cause);
        }
    }
}
// endregion
