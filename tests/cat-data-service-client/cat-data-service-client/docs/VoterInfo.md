# VoterInfo

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**voting_power** | Option<[**serde_json::Value**](.md)> | Voting keys voting power. This is the true voting power, subject to minimum voting power and max cap. | 
**voting_group** | [**crate::models::VoterGroupId**](VoterGroupId.md) |  | 
**delegations_power** | Option<[**serde_json::Value**](.md)> | Total voting power delegated to this voting key. This is not capped and not subject to minimum voting power. | 
**delegations_count** | Option<[**serde_json::Value**](.md)> | Number of registration which delegated to this voting key. | 
**voting_power_saturation** | Option<[**serde_json::Value**](.md)> | Voting power's share of the total voting power. Can be used to gauge potential voting power saturation. This value is NOT saturated however, and gives the raw share of total registered voting power. | 
**delegator_addresses** | Option<[**serde_json::Value**](.md)> | List of stake public key addresses which delegated to this voting key. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


