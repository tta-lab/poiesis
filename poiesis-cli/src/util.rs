use std::io::{self, Read};

/// Read all of stdin into a string
pub fn read_stdin() -> Result<String, io::Error> {
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf)?;
    Ok(buf)
}

/// Try to read stdin — returns None if stdin is a TTY (no piped input)
pub fn try_read_stdin() -> Result<Option<String>, io::Error> {
    use std::io::IsTerminal;
    if io::stdin().is_terminal() {
        Ok(None)
    } else {
        let content = read_stdin()?;
        if content.is_empty() {
            Ok(None)
        } else {
            Ok(Some(content))
        }
    }
}

/// Print an error to stderr and exit with code 1
pub fn fatal(msg: &str) -> ! {
    eprintln!("error: {}", msg);
    std::process::exit(1);
}

/// Display a PoiesisError to stderr and exit with code 1
pub fn fatal_err(err: &poiesis_core::PoiesisError) -> ! {
    eprintln!("error: {}", err);
    std::process::exit(1);
}
