# VoteInfo

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**fragment_id** | Option<[**serde_json::Value**](.md)> | Blockchain ID of the vote plan transaction | [optional]
**caster** | Option<[**serde_json::Value**](.md)> | public key of caster wallet | [optional]
**proposal** | Option<[**serde_json::Value**](.md)> | proposal index within voteplan | [optional]
**voteplan_id** | Option<[**serde_json::Value**](.md)> | Blockchain ID of the vote plan transaction | [optional]
**time** | Option<[**serde_json::Value**](.md)> | block date in format epoch.slot_no | [optional]
**choice** | Option<[**serde_json::Value**](.md)> | vote choice (only visible for public voting) | [optional]
**raw_fragment** | Option<[**serde_json::Value**](.md)> | raw bytes of transaction | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


