use game::Config;

pub type Error = ();

pub fn parse(_s: &str) -> Result<Config, Error> {
    // TODO real parsing
    Ok(Config::default())
}
