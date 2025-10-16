use game::Config;

pub type Error = ();

pub fn parse(_s: &str) -> Result<Config, Error> {
    Ok(Config{})
}
