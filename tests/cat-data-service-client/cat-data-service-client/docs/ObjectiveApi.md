# \ObjectiveApi

All URIs are relative to *http://localhost:8080*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_objectives**](ObjectiveApi.md#get_objectives) | **GET** /api/v1/event/{id}/objectives | Get objectives to be decided for a Voting Event.



## get_objectives

> serde_json::Value get_objectives(limit, offset, grp)
Get objectives to be decided for a Voting Event.

Get all current objectives to be decided for the voting event. An Objective here is defined as:\\ *\"A vote in which all the people in a group decide on an important issue.\"*\\ Examples of objectives are: * Catalyst Funding Challenges * General Voting Events * Any other Decision to be determined by the Communities Collective will. * Non binding community collective opinion measurement.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**limit** | Option<[**serde_json::Value**](.md)> | Limit the results to this value |  |[default to 2147483647]
**offset** | Option<[**serde_json::Value**](.md)> | Offset the results starting at this record. 0 being the first. |  |[default to 0]
**grp** | Option<[**serde_json::Value**](.md)> | Filter Results to only these voter groups. |  |

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

