# DelegatorInfo

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**delegations** | Option<[**serde_json::Value**](.md)> | List off delegations made by this stake address. In the order as presented in the voters registration. | [optional]
**reward_address** | Option<[**serde_json::Value**](.md)> | Hex encoded reward address for this delegation. | [optional]
**raw_power** | Option<[**serde_json::Value**](.md)> | Raw total voting power from stake address | [optional]
**total_power** | Option<[**serde_json::Value**](.md)> | Total voting power, across all registered voters. | [optional]
**last_updated** | Option<[**serde_json::Value**](.md)> | Date and time for the latest update to this snapshot information. | [optional]
**as_at** | Option<[**serde_json::Value**](.md)> | Date and time the latest snapshot represents. | [optional]
**r#final** | Option<[**serde_json::Value**](.md)> | `True` = this is the final snapshot which will be used for voting power in the event.</br>`False` =This is an interim snapshot, subject to change. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


