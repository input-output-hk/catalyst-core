use askama::Template;

#[derive(Template)]
#[template(path = "proposal_by_id.graphql.txt")]
pub struct ProposalById {
    pub id: u32,
}

#[derive(Template)]
#[template(path = "proposal_aliases.txt")]
pub struct ProposalAliases {
    pub id: u32,
}

#[derive(Template)]
#[template(path = "proposal_alias.txt")]
pub struct ProposalAlias {
    pub id: u32,
}
#[derive(Template)]
#[template(path = "proposal_comment.txt")]
pub struct ProposalComment {
    pub id: u32,
}

#[derive(Template)]
#[template(path = "proposal_cycle.txt")]
pub struct ProposalCycle {
    pub id: u32,
}

#[derive(Template)]
#[template(path = "proposal_field_does_not_exist.txt")]
pub struct ProposalDoesNotExist {
    pub id: u32,
}

#[derive(Template)]
#[template(path = "proposals.txt")]
pub struct Proposals;

#[derive(Template)]
#[template(path = "proposal_without_argument.txt")]
pub struct ProposalWithoutArgument;

#[derive(Template)]
#[template(path = "proposal_fragments.txt")]
pub struct ProposalFragments {
    pub id: u32,
}

#[derive(Template)]
#[template(path = "proposal_required_fields.txt")]
pub struct ProposalRequiredField {
    pub id: u32,
}
