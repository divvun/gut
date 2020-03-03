use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "dadmin", about = "git multirepo maintenance tool")]
pub enum Args {
    #[structopt(name = "init")]
    Init(InitArgs),
    #[structopt(name = "update-config")]
    Update(ConfigArgs),
}

#[derive(Debug, StructOpt)]
pub struct InitArgs {
    #[structopt(long, short, default_value = "./dadmin")]
    pub root: String,

    #[structopt(short, long)]
    pub token: String,
}

#[derive(Debug, StructOpt)]
pub struct ConfigArgs {
    #[structopt(long, short, default_value = "./dadmin")]
    pub root: String,
}
