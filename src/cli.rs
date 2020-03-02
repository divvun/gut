use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "dadmin", about = "git multirepo maintenance tool")]
pub enum Args {
    #[structopt(name = "init-config")]
    Init(ConfigArgs),
    #[structopt(name = "update-config")]
    Update(ConfigArgs),
}

#[derive(Debug, StructOpt)]
pub struct ConfigArgs {
    #[structopt(long, short, default_value = "./dadmin")]
    pub root: String,

    #[structopt(short, long)]
    pub name: String,

    #[structopt(short, long)]
    pub email: String,
}
