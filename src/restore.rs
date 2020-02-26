use super::args::Restore;

pub (crate) fn restore_main(parsed: Restore) -> Result<(), Box<dyn std::error::Error>> {

    if parsed.options.verbose {
        // TODO: use a logger and combine with --verbose
        println!("  {:?}", parsed);
    }

    Ok(())
}
