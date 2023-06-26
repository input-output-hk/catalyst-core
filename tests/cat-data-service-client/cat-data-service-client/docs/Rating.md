# Rating

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**review_type** | Option<[**serde_json::Value**](.md)> | The review type being rated. Maps to the ReviewType id. | 
**score** | Option<[**serde_json::Value**](.md)> | Score given to this rating. Will be bounded by the `min` and `max` of the ReviewType. | 
**note** | Option<[**serde_json::Value**](.md)> | Reason why this rating was given. If NO reason was given, this field is omitted. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


