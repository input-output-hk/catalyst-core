# \EventApi

All URIs are relative to *http://localhost:8080*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_event**](EventApi.md#get_event) | **GET** /api/v1/event/{id} | Get Individual Voting Event Details
[**get_events**](EventApi.md#get_events) | **GET** /api/v1/events | Get Voting Events.



## get_event

> serde_json::Value get_event(id)
Get Individual Voting Event Details

Retrieves all information about the requested Voting Event. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | [**serde_json::Value**](.md) | The ID Number of the Voting Event. As reported by `/api/v1/events`. | [required] |

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_events

> serde_json::Value get_events(limit, offset)
Get Voting Events.

Get list of all currently known past, present and future Voting Events. 

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

