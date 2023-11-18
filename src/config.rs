use clap::Parser;

#[derive(Parser, Debug)]
pub struct Config {
    #[clap(long, env)]
    pub database_url: String,

    #[clap(long, env)]
    pub openai_key: String,
}