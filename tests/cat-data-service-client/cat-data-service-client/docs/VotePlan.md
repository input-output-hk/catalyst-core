# VotePlan

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**chain_proposal_index** | Option<[**serde_json::Value**](.md)> | The Index of the proposal, needed to create a ballot for it. | 
**group** | [**crate::models::VoterGroupId**](VoterGroupId.md) | The name of the group (Must be unique in the array). | 
**ballot_type** | Option<[**crate::models::BallotType**](BallotType.md)> | The type of ballot this group must cast. | [optional]
**chain_voteplan_id** | Option<[**serde_json::Value**](.md)> | Blockchain ID of the vote plan transaction. | 
**encryption_key** | Option<[**serde_json::Value**](.md)> | The public encryption key used. ONLY if required by the ballot type (private, cast-private). | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


