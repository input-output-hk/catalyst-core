# VotersInfo

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**voter_info** | [**crate::models::VoterInfo**](VoterInfo.md) |  | 
**last_updated** | Option<[**serde_json::Value**](.md)> | Date and time for the latest update to this snapshot information. | 
**as_at** | Option<[**serde_json::Value**](.md)> | Date and time the latest snapshot represents. | [optional]
**r#final** | Option<[**serde_json::Value**](.md)> | `True` = this is the final snapshot which will be used for voting power in the event.</br>`False` =This is an interim snapshot, subject to change. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


