use color_eyre::Report;
use clap::Parser;

mod node;
mod vit_ss;

#[derive(Debug, Parser)]
#[clap(rename_all = "kebab-case")]
pub enum Archive {
    Node(node::Node),
    VitSS(vit_ss::VitSS),
}

impl Archive {
    pub fn exec(self) -> Result<(), Report> {
        match self {
            Archive::Node(node) => node.exec(),
            Archive::VitSS(vit_ss) => vit_ss.exec(),
        }
    }
}
