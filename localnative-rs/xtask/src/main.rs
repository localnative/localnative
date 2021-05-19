pub mod flags;
mod release;

use anyhow::Result;

fn main() -> Result<()> {
    let cmd = flags::Xtask::from_env()?;
    match cmd.subcommand {
        flags::XtaskCmd::Help(_) => {
            println!("{}", flags::Xtask::HELP);
        }
        flags::XtaskCmd::Release(cmd) => cmd.run()?,
    }
    Ok(())
}
