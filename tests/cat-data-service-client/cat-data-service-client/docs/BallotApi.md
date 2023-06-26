# \BallotApi

All URIs are relative to *http://localhost:8080*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_all_ballot_info_per_event**](BallotApi.md#get_all_ballot_info_per_event) | **GET** /api/v1/event/{id}/ballots | Get all ballot information needed to cast a vote associated with an event
[**get_all_ballot_info_per_objective**](BallotApi.md#get_all_ballot_info_per_objective) | **GET** /api/v1/event/{id}/objective/{obj_id}/ballots | Get all ballot information needed to cast a vote associated with an objective
[**get_api_v1_ballot_cast_id**](BallotApi.md#get_api_v1_ballot_cast_id) | **GET** /api/v1/ballot/cast/{id} | Cast a ballot
[**get_ballot_info_per_proposal**](BallotApi.md#get_ballot_info_per_proposal) | **GET** /api/v1/event/{id}/objective/{obj_id}/proposal/{prop_id}/ballot | Get ballot information needed to cast a vote associated with a proposal
[**get_ballot_proofs**](BallotApi.md#get_ballot_proofs) | **POST** /api/v1/ballot/proof/{vkey}/{id} | Get Ballot Proof
[**post_api_v1_ballot_check**](BallotApi.md#post_api_v1_ballot_check) | **POST** /api/v1/ballot/check | Check Ballot Validity



## get_all_ballot_info_per_event

> serde_json::Value get_all_ballot_info_per_event(id)
Get all ballot information needed to cast a vote associated with an event

Retrieves all necessary information to cast a vote. If not present, they are not yet defined. If not defined, then it is not possible to cast a ballot on this proposal yet. 

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


## get_all_ballot_info_per_objective

> serde_json::Value get_all_ballot_info_per_objective(id, obj_id)
Get all ballot information needed to cast a vote associated with an objective

Retrieves all necessary information to cast a vote. If not present, they are not yet defined. If not defined, then it is not possible to cast a ballot on this proposal yet. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | [**serde_json::Value**](.md) | The ID Number of the Voting Event. As reported by `/api/v1/events`. | [required] |
**obj_id** | [**serde_json::Value**](.md) | The ID number of the Objective in a Voting Event. As reported by `/api/v1/event/{id}/objectives`. | [required] |

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_api_v1_ballot_cast_id

> get_api_v1_ballot_cast_id(id)
Cast a ballot

Cast a Ballot on the requested Voting Event.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | [**serde_json::Value**](.md) | The ID Number of the Voting Event. As reported by `/api/v1/events`. | [required] |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_ballot_info_per_proposal

> crate::models::CatalystV1Ballot get_ballot_info_per_proposal(id, obj_id, prop_id)
Get ballot information needed to cast a vote associated with a proposal

Retrieves all necessary information to cast a vote. If not present, they are not yet defined. If not defined, then it is not possible to cast a ballot on this proposal yet. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | [**serde_json::Value**](.md) | The ID Number of the Voting Event. As reported by `/api/v1/events`. | [required] |
**obj_id** | [**serde_json::Value**](.md) | The ID number of the Objective in a Voting Event. As reported by `/api/v1/event/{id}/objectives`. | [required] |
**prop_id** | [**serde_json::Value**](.md) | The ID Number of the Proposal within a Objective of a Voting Event | [required] |

### Return type

[**crate::models::CatalystV1Ballot**](CatalystV1Ballot.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_ballot_proofs

> crate::models::VoteInfo get_ballot_proofs(vkey, id, votes_by_vote_caster_and_voteplan_id)
Get Ballot Proof

Get list of details and ballot proofs for every currently cast ballot on a particular voting event for a particular voting key.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**vkey** | [**serde_json::Value**](.md) | A Voting Key. | [required] |
**id** | [**serde_json::Value**](.md) | The ID Number of the Voting Event. As reported by `/api/v1/events`. | [required] |
**votes_by_vote_caster_and_voteplan_id** | [**VotesByVoteCasterAndVoteplanId**](VotesByVoteCasterAndVoteplanId.md) | List of votes by voteplan id and caster (wallet) | [required] |

### Return type

[**crate::models::VoteInfo**](VoteInfo.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_api_v1_ballot_check

> post_api_v1_ballot_check()
Check Ballot Validity

Check if a Ballot is correctly formatted and signed. This endpoint is to check all possible aspects of an individual ballot and return a report details what is correct, and what is incorrect. The actual format of the report is TBD on implementation. It is to handle both Private and Public Ballots.  If Event ID is provided, the ballot should also be checked against current knowledge of the Events Objectives and Proposals to check the contents of the ballot are correct.  Otherwise, they are not considered and ONLY the raw format is checked.  It must also check the Ballot is signed properly with the voters private voting key.  If an event is supplied, it should also check if the Voting Key is registered and eligible to vote on the event. If no event is supplied, it should check if the \"latest\" snapshot contains a registration for this voting key.  If it does not, this is not an Error, but a warning status should be returned.

### Parameters

This endpoint does not need any parameter.

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

