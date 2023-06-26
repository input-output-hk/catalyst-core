# ProposalDetails

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**funds** | Option<[**serde_json::Value**](.md)> | The amount of funds requested by this proposal. In the denomination of the Objectives Reward. If not present, then this proposal is not requesting any funds. | [optional]
**url** | Option<[**serde_json::Value**](.md)> | URL to a web page with details on this proposal. | [optional]
**files** | Option<[**serde_json::Value**](.md)> | Link to extra files associated with this proposal. Only included if there are linked files. | [optional]
**proposer** | Option<[**serde_json::Value**](.md)> | List of all proposers making this proposal. | [optional]
**supplemental** | Option<[**crate::models::ProposalSupplementalDetails**](ProposalSupplementalDetails.md)> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


