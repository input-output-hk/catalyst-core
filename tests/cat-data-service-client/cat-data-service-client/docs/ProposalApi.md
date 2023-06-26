# \ProposalApi

All URIs are relative to *http://localhost:8080*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_all_proposals**](ProposalApi.md#get_all_proposals) | **GET** /api/v1/event/{id}/objective/{obj_id}/proposals | Get summary all available proposals
[**get_proposal**](ProposalApi.md#get_proposal) | **GET** /api/v1/event/{id}/objective/{obj_id}/proposal/{prop_id} | Get Individual Proposal data



## get_all_proposals

> serde_json::Value get_all_proposals(limit, offset, grp)
Get summary all available proposals

Summarized Lists all available proposals. 

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


## get_proposal

> serde_json::Value get_proposal(id, obj_id, prop_id)
Get Individual Proposal data

Retrieves information on the identified proposal if it belongs to the provided group. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | [**serde_json::Value**](.md) | The ID Number of the Voting Event. As reported by `/api/v1/events`. | [required] |
**obj_id** | [**serde_json::Value**](.md) | The ID number of the Objective in a Voting Event. As reported by `/api/v1/event/{id}/objectives`. | [required] |
**prop_id** | [**serde_json::Value**](.md) | The ID Number of the Proposal within a Objective of a Voting Event | [required] |

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

