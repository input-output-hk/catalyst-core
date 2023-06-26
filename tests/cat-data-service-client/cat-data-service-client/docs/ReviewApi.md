# \ReviewApi

All URIs are relative to *http://localhost:8080*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_all_review_types**](ReviewApi.md#get_all_review_types) | **GET** /api/v1/event/{id}/objective/{obj_id}/review_types | Get summary all available review types for this objective.
[**get_proposal_reviews**](ReviewApi.md#get_proposal_reviews) | **GET** /api/v1/event/{id}/objective/{obj_id}/proposal/{prop_id}/reviews | Get reviews related to a proposal



## get_all_review_types

> serde_json::Value get_all_review_types(id, obj_id, limit, offset)
Get summary all available review types for this objective.

Lists all available review types used on this events proposal. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | [**serde_json::Value**](.md) | The ID Number of the Voting Event. As reported by `/api/v1/events`. | [required] |
**obj_id** | [**serde_json::Value**](.md) | The ID number of the Objective in a Voting Event. As reported by `/api/v1/event/{id}/objectives`. | [required] |
**limit** | Option<[**serde_json::Value**](.md)> | Limit the results to this value |  |[default to 2147483647]
**offset** | Option<[**serde_json::Value**](.md)> | Offset the results starting at this record. 0 being the first. |  |[default to 0]

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_proposal_reviews

> serde_json::Value get_proposal_reviews(limit, offset)
Get reviews related to a proposal

Retrieves advisor reviews information for the provided proposal id. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**limit** | Option<[**serde_json::Value**](.md)> | Limit the results to this value |  |[default to 2147483647]
**offset** | Option<[**serde_json::Value**](.md)> | Offset the results starting at this record. 0 being the first. |  |[default to 0]

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

