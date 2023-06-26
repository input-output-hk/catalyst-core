# \RegistrationApi

All URIs are relative to *http://localhost:8080*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_delegator_info**](RegistrationApi.md#get_delegator_info) | **GET** /api/v1/registration/delegations/{sp_key}?event_id={eid} | Get voters info by stake public key
[**get_voter_info**](RegistrationApi.md#get_voter_info) | **GET** /api/v1/registration/voter/{vkey}?event_id={eid}&with_delegators={flag} | Get voter's info by voting key



## get_delegator_info

> crate::models::DelegatorInfo get_delegator_info(sp_key, event_id)
Get voters info by stake public key

Get voters delegation info by stake public key.\\ If the `eid` query parameter is missing, then the \"Latest\" registration for the Stake Public Key is returned. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**sp_key** | [**serde_json::Value**](.md) | Stake Public key | [required] |
**event_id** | Option<[**serde_json::Value**](.md)> | The Event ID to return results for. |  |

### Return type

[**crate::models::DelegatorInfo**](DelegatorInfo.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_voter_info

> crate::models::VotersInfo get_voter_info(vkey, event_id, with_delegators)
Get voter's info by voting key

Get voter's registration and voting power by their voting key.\\ If the `event_id` query parameter is omitted, then the latest voting power is retrieved. If the `with_delegators` query parameter is ommitted, then `delegator_addresses` field of `VoterInfo` type does not provided. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**vkey** | [**serde_json::Value**](.md) | A Voting Key. | [required] |
**event_id** | Option<[**serde_json::Value**](.md)> | The Event ID to return results for. |  |
**with_delegators** | Option<[**serde_json::Value**](.md)> | Flag to include delegators list in the response. |  |

### Return type

[**crate::models::VotersInfo**](VotersInfo.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

