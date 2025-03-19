use super::fails::{BoxedError, BoxedResult, raise};
use super::{args::Cli, args::Generate};
use clap::CommandFactory;
use clap_complete::{Generator, Shell, generate};
use std::{fs::File, io, io::Write, path::PathBuf};

pub(crate) fn gen_assets(params: &Generate) -> BoxedError {
    let chosen = params.get_shells();
    for shell in chosen {
        let path = generate_for(&shell, &params.output_dir)?;
        if let Some(filepath) = path {
            println!("{:?}: {}", shell, filepath.display());
        }
    }
    if params.manpage || params.all {
        let manpath = generate_manpage(&params.output_dir)?;
        if let Some(mpath) = manpath {
            println!("Manpage: {}", mpath.display());
        }
    }
    Ok(())
}

fn generate_manpage(output_dir: &Option<PathBuf>) -> BoxedResult<Option<PathBuf>> {
    let app = Cli::command().get_name().to_string();
    let cmd = Cli::command();
    if let Some(dir) = output_dir {
        let manpath = dir.join(&app).with_extension("1");

        let man = clap_mangen::Man::new(cmd);
        let mut buffer: Vec<u8> = Default::default();
        man.render(&mut buffer)?;

        std::fs::write(manpath.clone(), buffer)?;

        return Ok(Some(manpath));
    }
    raise("No output directory specified to output manpage")
}

fn generate_for(shell: &Shell, output_dir: &Option<PathBuf>) -> BoxedResult<Option<PathBuf>> {
    let app = Cli::command().get_name().to_string();
    let mut cmd = Cli::command();

    let path = output_dir.clone().map(|dir| dir.join(shell.file_name(&app)));

    let mut file: Box<dyn Write> = match path.clone() {
        None => Box::new(io::stdout()),
        Some(dir) => Box::new(File::create(&dir)?),
    };
    generate(*shell, &mut cmd, app, &mut file);
    Ok(path)
}

// end of file \\
