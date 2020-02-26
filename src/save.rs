use zip::write::FileOptions;
use zip::result::ZipResult;
use zip::ZipWriter;

use std::io::Write;

use chrono::{DateTime, Utc};

use super::args::*;
use super::fails::*;
use super::steps::*;

// region Archiver 

impl Arguments {

    pub fn get_writer(&self) -> Result<Archiver, BoxedError> {

        let name = match &self.name {
            Some(text) => text,
            None => &self.from,
        };
        let res = Archiver::write_on(&self.into, &name);
        Ok(res)
    }
}

type Compressor = ZipWriter<std::fs::File>;

pub struct Archiver {
    writer: Option<Compressor>,
    folder: std::path::PathBuf,
    name: String,
    sequence: u64,
}

impl Archiver {

    fn write_on(dir: &std::path::PathBuf, core_name: &str) -> Self {
        let now: DateTime<Utc> = Utc::now();
        let time = now.format("%Y-%m-%d_%H-%M-%S");
        let name = format!("{}_{}", core_name, time);
        Archiver { 
            writer: None,
            folder: dir.to_owned(),
            name: name,
            sequence: 0,
        }
    }

    pub fn create_archive(&mut self) -> ZipResult<()> {

        self.close_archive()?;

        self.sequence +=1;
        let file_name = format!("{}_{:05}.zip", &self.name, &self.sequence);
        let zip_path = std::path::Path::new(&self.folder);
        let zip_name = std::path::Path::new(&file_name);
        let zip_file = zip_path.join(&zip_name);

        let file = std::fs::File::create(&zip_file)?;
        let zip = zip::ZipWriter::new(file);

        self.writer = Some(zip);
        Ok(())
    }

    pub fn write_file(&mut self, step: &Step, rows: &str) -> ZipResult<()> {

        if self.writer.is_none() {
            self.create_archive()?;
        }
        let filename = format!("rows_at_{:09}.json", &step.curr);
        let bytes = rows.as_bytes();

        let zip = self.writer.as_mut().unwrap();

        let opts: FileOptions = FileOptions::default()
                .compression_method(zip::CompressionMethod::Stored)
                .unix_permissions(0o644);

        zip.start_file(filename, opts)?;
        zip.write_all(bytes)?;
        zip.flush()?;
    
        // TODO: limit file size based on zip.stats.bytes_written
        
        Ok(())
    }

    pub fn close_archive(&mut self) -> ZipResult<()> {

        if self.writer.is_some() {
            self.writer.as_mut().unwrap()
                .finish()?;
        }
        self.writer = None;
        Ok(())
    }
}

// endregion
