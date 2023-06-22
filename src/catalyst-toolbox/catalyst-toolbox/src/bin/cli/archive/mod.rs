use clap::Parser;
use color_eyre::Report;

mod node;

#[derive(Debug, Parser)]
#[clap(rename_all = "kebab-case")]
pub enum Archive {
    Node(node::Node),
}

impl Archive {
    pub fn exec(self) -> Result<(), Report> {
        match self {
            Archive::Node(node) => node.exec(),
        }
    }
}
