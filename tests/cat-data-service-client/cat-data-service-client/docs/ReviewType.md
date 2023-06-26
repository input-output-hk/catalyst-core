# ReviewType

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | Option<[**serde_json::Value**](.md)> | The Unique ID for this review type. | 
**name** | Option<[**serde_json::Value**](.md)> | The unique name for the review type. | 
**description** | Option<[**serde_json::Value**](.md)> | Description about what the review type is. | [optional]
**min** | Option<[**serde_json::Value**](.md)> | The inclusive Minimum value for the reviews rating. By definition, lower value ratings are considered lower ratings. Therefore this field represents the lowest possible rating. | [default to 0]
**max** | Option<[**serde_json::Value**](.md)> | The inclusive Maximum value for the reviews rating. By definition, higher value ratings are considered higher ratings. Therefore this field represents the highest possible rating. | 
**note** | Option<[**serde_json::Value**](.md)> | Does the Review Type include a note? * Null - *Optional*, may or may not include a note. * False - **MUST NOT** include a note. * True - **MUST** include a note. | [optional]
**map** | Option<[**serde_json::Value**](.md)> | Optional sequential list of mapped named values for rating scores. * If not present, the rating score is numeric. * If present:   * all possible rating scores must be represented with mapped names and the rating is represented by the value in the map.   * The lowest numbered score comes first in the array.   * The array is sequential with no gaps. | 
**group** | Option<[**serde_json::Value**](.md)> | The reviewer group who can create this review type. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


