# \SearchApi

All URIs are relative to *http://localhost:8080*

Method | HTTP request | Description
------------- | ------------- | -------------
[**search**](SearchApi.md#search) | **POST** /api/v1/search | Search various resources with various constraints



## search

> crate::models::Search200Response search(limit, offset, total, search_query)
Search various resources with various constraints

Search various resources especially challenges and proposals with various constraints like contains some string etc.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**limit** | Option<[**serde_json::Value**](.md)> | Limit the results to this value |  |[default to 2147483647]
**offset** | Option<[**serde_json::Value**](.md)> | Offset the results starting at this record. 0 being the first. |  |[default to 0]
**total** | Option<[**serde_json::Value**](.md)> | Don't return results, just the total number of results that could have been returned. |  |[default to false]
**search_query** | Option<[**SearchQuery**](SearchQuery.md)> | Parameters to the search. |  |

### Return type

[**crate::models::Search200Response**](search_200_response.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

