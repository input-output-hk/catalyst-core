# VotingPowerSettings

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**alg** | Option<[**serde_json::Value**](serde_json::Value.md)> | The Voting Power Algorithm.  * `threshold_staked_ADA` = \"Linear Voting Power in Staked ADA, With a minimum limit and maximum relative threshold. | 
**min_ada** | Option<[**serde_json::Value**](.md)> | Minimum staked funds required for a valid voter registration. This amount is in Whole ADA. If not present, there is no minimum.\\ Valid for `alg`: * `threshold_staked_ADA` | [optional]
**max_pct** | Option<[**serde_json::Value**](.md)> | Maximum Percentage of total registered voting power allowed for voting power. For example `1.23` = `1.23%` of total registered staked ADA as maximum voting power. If not present, there is no maximum percentage.\\ Valid for `alg`: * `threshold_staked_ADA` | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


