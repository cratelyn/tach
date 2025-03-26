//! a compact cpu monitor.

type Error = Box<dyn std::error::Error>;

fn main() -> Result<(), Error> {
    let app = tach::App::new();
    app.run()?;
    unreachable!();
}
