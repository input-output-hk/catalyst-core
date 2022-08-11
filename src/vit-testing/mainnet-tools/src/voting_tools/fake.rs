use catalyst_toolbox::snapshot::registration::Delegations;
use catalyst_toolbox::snapshot::VotingRegistration;
use jormungandr_lib::crypto::account::Identifier;

pub fn default() -> Vec<VotingRegistration> {
    vec![
        VotingRegistration {
            reward_address: "0xe1ffff2912572257b59dca84c965e4638a09f1524af7a15787eb0d8a46"
                .to_string(),
            stake_public_key: "0xe7d6616840734686855ec80ee9658f5ead9e29e494ec6889a5d1988b50eb8d0f"
                .to_string(),
            voting_power: 177689370111.into(),
            delegations: Delegations::Legacy(
                Identifier::from_hex(
                    "0xc21ddb4abb04bd5ce21091eef1676e44889d806e6e1a6a9a7dc25c0eba54cc33",
                )
                .unwrap(),
            ),
            voting_purpose: 0,
        },
        VotingRegistration {
            reward_address: "0xe1fffc8bcb1578a15413bf11413639fa270a9ffa36d9a0c4d2c93536fe"
                .to_string(),
            stake_public_key: "0x2f9a90d87321a255efd038fea5df2a2349ea2c32fa584b73f2a46f655f235919"
                .to_string(),
            voting_power: 9420156337.into(),
            delegations: Delegations::Legacy(
                Identifier::from_hex(
                    "0x3f656a1ba4ea8b33c81961fee6f15f09600f024435b1a7ada1e5b77b03a41a6d",
                )
                .unwrap(),
            ),
            voting_purpose: 0,
        },
        VotingRegistration {
            reward_address: "0xe1fff825e1bf009d35d9160f6340250b581f5d37c17538e960c0410b20"
                .to_string(),
            stake_public_key: "0x66ae1553036548b99b93c783811bb281be5a196a12d950bda4ac9b83630afbd1"
                .to_string(),
            voting_power: 82168168290u64.into(),
            delegations: Delegations::Legacy(
                Identifier::from_hex(
                    "0x125860fc4870bb480d1d2a97f101e1c5c845c0222400fdaba7bcca93e79bd66e",
                )
                .unwrap(),
            ),
            voting_purpose: 0,
        },
    ]
}
