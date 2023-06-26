# VitSsFund

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | Option<[**serde_json::Value**](.md)> | Identifier of the fund campaign. | [optional]
**fund_name** | Option<[**serde_json::Value**](.md)> | Human-readable name of the fund campaign. | [optional]
**fund_goal** | Option<[**serde_json::Value**](.md)> | Description of the campaign's goals. | [optional]
**voting_power_info** | Option<[**serde_json::Value**](.md)> | Deprecated, same as registration_snapshot_time. | [optional]
**voting_power_threshold** | Option<[**serde_json::Value**](.md)> | Minimal amount of funds required for a valid voter registration. This amount is in lovelace.  | [optional]
**rewards_info** | Option<[**serde_json::Value**](.md)> |  | [optional]
**fund_start_time** | Option<[**serde_json::Value**](.md)> | Date and time for the start of the current voting period. | [optional]
**fund_end_time** | Option<[**serde_json::Value**](.md)> | Date and time for the end of the current voting period. | [optional]
**next_fund_start_time** | Option<[**serde_json::Value**](.md)> | Date and time for the start of the next voting period. | [optional]
**registration_snapshot_time** | Option<[**serde_json::Value**](.md)> | Date and time for blockchain state snapshot capturing voter registrations. | [optional]
**next_registration_snapshot_time** | Option<[**serde_json::Value**](.md)> | Date and time for next blockchain state snapshot capturing voter registrations. | [optional]
**chain_vote_plans** | Option<[**serde_json::Value**](.md)> | Vote plans registered for voting in this fund campaign. | [optional]
**groups** | Option<[**serde_json::Value**](.md)> |  | [optional]
**challenges** | Option<[**serde_json::Value**](.md)> | A list of campaign challenges structuring the proposals. | [optional]
**goals** | Option<[**serde_json::Value**](.md)> | The list of campaign goals for this fund. | [optional]
**insight_sharing_start** | Option<[**serde_json::Value**](.md)> |  | [optional]
**proposal_submission_start** | Option<[**serde_json::Value**](.md)> |  | [optional]
**refine_proposals_start** | Option<[**serde_json::Value**](.md)> |  | [optional]
**finalize_proposals_start** | Option<[**serde_json::Value**](.md)> |  | [optional]
**proposal_assessment_start** | Option<[**serde_json::Value**](.md)> |  | [optional]
**assessment_qa_start** | Option<[**serde_json::Value**](.md)> |  | [optional]
**snapshot_start** | Option<[**serde_json::Value**](.md)> |  | [optional]
**voting_start** | Option<[**serde_json::Value**](.md)> |  | [optional]
**voting_end** | Option<[**serde_json::Value**](.md)> |  | [optional]
**tallying_end** | Option<[**serde_json::Value**](.md)> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


