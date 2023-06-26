# ProposerDetails

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**name** | Option<[**serde_json::Value**](.md)> | Name of the author/s of the proposal. | [optional]
**email** | Option<[**serde_json::Value**](.md)> | Email contact address of the author/s of the proposal. If not present, there is no known contact email for the authors. | [optional]
**url** | Option<[**serde_json::Value**](.md)> | URL to a web resource with details about the author/s of the proposal. | [optional]
**payment_key** | Option<[**serde_json::Value**](.md)> | The Payment Address the Funds requested will be paid to. Will not be included if the proposal does not request funds. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


