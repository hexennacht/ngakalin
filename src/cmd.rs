use structopt::StructOpt;

#[derive(Debug, StructOpt, Clone)]
#[structopt(name="ngakalin", about = "how to use ngakalin")]
pub struct Command {
    #[structopt(short="c", long="config", default_value="configuration.yaml")]
    pub config: String
}