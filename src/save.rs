use log::error;
use zip::{result::ZipResult, write::SimpleFileOptions, ZipWriter};

use std::{
    io::Write,
    path::{Path, PathBuf},
};

use crate::steps::Documents;

// TODO: split in multiple files of constant size
// TODO: limit file size based on zip.stats.bytes_written

// region Archiver

type Compressor = ZipWriter<std::fs::File>;

pub struct Archiver {
    writer: Option<Compressor>,
    folder: PathBuf,
    file_pattern: String,
    max_files: usize,
    file_count: usize,
}

impl Archiver {
    pub fn write_on(output_dir: &Path, output_pattern: &str, max: usize) -> Self {
        Archiver {
            writer: None,
            folder: output_dir.to_owned(),
            file_pattern: output_pattern.to_string(),
            max_files: max,
            file_count: 0,
        }
    }

    fn create_archive(&mut self, suffix: &str) -> ZipResult<()> {
        self.close_archive()?;

        let file_name = self.file_pattern.replace("{}", suffix);
        let zip_path = Path::new(&self.folder);
        let zip_name = Path::new(&file_name);
        let zip_file = zip_path.join(&zip_name);

        let file = std::fs::File::create(&zip_file)?;
        let zip = zip::ZipWriter::new(file);

        self.writer = Some(zip);
        self.file_count = 0;
        Ok(())
    }

    fn write_file(&mut self, filename: &str, docs: &str) -> ZipResult<()> {
        let bytes = docs.as_bytes();

        let zip = self.writer.as_mut().unwrap();

        let opts = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(0o644);

        zip.start_file(filename, opts)?;
        zip.write_all(bytes)?;
        zip.flush()?;
        Ok(())
    }

    pub fn close_archive(&mut self) -> ZipResult<()> {
        if let Some(wr) = self.writer.take() {
            wr.finish()?;
        }
        self.writer = None;
        Ok(())
    }

    pub fn write_documents(&mut self, docs: &Documents) -> ZipResult<()> {
        let json = &docs.docs;
        let step = &docs.step;

        let filename = format!("docs_at_{:09}.json", step.curr + 1);

        self.file_count += 1;
        let wrap = self.file_count >= self.max_files;

        if self.writer.is_none() || wrap {
            let suffix = format!("{:09}", step.curr + 1);
            self.create_archive(&suffix)?;
        }
        self.write_file(&filename, json)
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
