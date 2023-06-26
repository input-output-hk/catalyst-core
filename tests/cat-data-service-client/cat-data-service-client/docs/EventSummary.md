# EventSummary

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | Option<[**serde_json::Value**](.md)> | The Numeric ID of a Voting Event | 
**name** | Option<[**serde_json::Value**](.md)> | The Name of a Voting Event | 
**starts** | Option<[**serde_json::Value**](.md)> | RFC3339 DateTime String UTC. | [optional]
**ends** | Option<[**serde_json::Value**](.md)> | RFC3339 DateTime String UTC. | [optional]
**r#final** | Option<[**serde_json::Value**](.md)> | True if the event is finished and no changes can be made to it.<br>Does not Including payment of rewards or funding of projects. | 
**reg_checked** | Option<[**serde_json::Value**](.md)> | RFC3339 DateTime String UTC. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


