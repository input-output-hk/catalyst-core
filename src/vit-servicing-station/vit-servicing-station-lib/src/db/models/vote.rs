use crate::db::schema::votes;
use diesel::{ExpressionMethods, Insertable, Queryable};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Queryable)]
pub struct Vote {
    pub fragment_id: String,
    pub caster: String,
    pub proposal: i32,
    pub voteplan_id: String,
    pub time: f32,
    pub choice: Option<i16>,
    pub raw_fragment: String,
}

impl Insertable<votes::table> for Vote {
    #[allow(clippy::type_complexity)]
    type Values = (
        diesel::dsl::Eq<votes::fragment_id, String>,
        diesel::dsl::Eq<votes::caster, String>,
        diesel::dsl::Eq<votes::proposal, i32>,
        diesel::dsl::Eq<votes::voteplan_id, String>,
        diesel::dsl::Eq<votes::time, f32>,
        diesel::dsl::Eq<votes::choice, Option<i16>>,
        diesel::dsl::Eq<votes::raw_fragment, String>,
    );

    fn values(self) -> Self::Values {
        (
            votes::fragment_id.eq(self.fragment_id),
            votes::caster.eq(self.caster),
            votes::proposal.eq(self.proposal),
            votes::voteplan_id.eq(self.voteplan_id),
            votes::time.eq(self.time),
            votes::choice.eq(self.choice),
            votes::raw_fragment.eq(self.raw_fragment),
        )
    }
}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::db::DbConnectionPool;
    use diesel::RunQueryDsl;

    pub fn get_test_vote() -> Vote {
        Vote {
                        fragment_id: "387797d0470ff23f1029c56556ac17dc4721d9da8b64d691de5029a8634906db".to_string(),
                        caster: "ca1q4t7y93d3z2szyapg4n59xyepcspn66agepfmu2l70xp72dalylt6gc5k9k".to_string(),
                        voteplan_id: "936e034e5c3723029934526c7f8118d257f7036c224e4b44e31f8bb44c1faccc".to_string(),
                        choice: None,
                        raw_fragment: "936e034e5c3723029934526c7f8118d257f7036c224e4b44e31f8bb44c1faccc1802024ccc9c19216b67674612f13ad2567cddee99a9839c82e12f67938afbde55da2afe422242c8e8b10d3073b6601455085feaa38e5ccf68fb69503195e385ff32444ccc9c19216b67674612f13ad2567cddee99a9839c82e12f67938afbde55da2ad402cf6b7185f88d8aab2be516481d521f1f21e3fac5be1bae1227505dd7ae0c01226b872c77446c76fabe9647467cbf305546470ce13706ad81b838ebc48f527fe67035ca0f136aefb761b7dbfa15c4cc2320541c1461234cbdd3c618ce6adf069c558281cad980de790c59c55c5257c1fd70b7ac225db1c55a8131cf1bb07278484412346dee1c6f1d31777901f1abf43f67e60800512fe80372e2df9f513911f82dcc9de90c8852d3b5d5c2dbe25462d54878746550f659696c73ddf77bb9406151d32fa56b44e5ecd6c82f5596d546b3c15d6ce05ca65f913c99a880c8f70caf3fa0d09c32116b460d3ebc3275493596db84257bb41bc0a3ecdb36d9b7f4021640660431463d76a69a1a95345ddc6e5eb6a15f22a4fe0673c00d4dc3f50a0f0f9107b937d1689217ec2d0ea730749e942bf4da659b596fd44bcddfee38c900000000070000005f0100ff000000000000000057e2162d88950113a145674298990e2019eb5d46429df15ff3cc1f29bdf93ebd02f3365ca3b87549eb3cf84e0cb60877cd4231b2e632d3108c087b56002982506322a3013924c08520b84d821c96d4ac896180ca926fea91a9f3c2d2e2b067f700".to_string(),
                        proposal: 24,
                        time: 0.1,
                    }
    }

    pub fn populate_db_with_vote(vote: &Vote, pool: &DbConnectionPool) {
        let connection = pool.get().unwrap();

        diesel::insert_into(votes::table)
            .values(vote.clone().values())
            .execute(&connection)
            .unwrap();
    }
}
