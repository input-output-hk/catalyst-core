import "encoding/json"

project: {
	name: "voting-node"

	deployment: {
		on: {
			//merge: {}
			//tag: {}
			always: {}
		}

		bundle: {
			env: string | *"dev"
			modules: main: {
				name:    "app"
				version: "0.11.1"
				values: {
					deployment: {
						containers: main: {
							image: {
								name: _ @forge(name="CONTAINER_IMAGE")
								tag:  _ @forge(name="GIT_HASH_OR_TAG")
							}
							env: {
								TESTNET_DBSYNC_URL: {
									secret: {
										name: "preprod-url"
										key:  "url"
									}
								}
								MAINNET_DBSYNC_URL: {
									secret: {
										name: "mainnet-url"
										key:  "url"
									}
								}
								EVENTDB_URL: {
									secret: {
										name: "eventdb-url"
										key:  "url"
									}
								}
								COMMITTEE_CRS: {
									secret: {
										name: "generated"
										key:  "crs"
									}
								}
								SECRET_SECRET: {
									secret: {
										name: "generated"
										key:  "secret"
									}
								}
								DBSYNC_SSH_PRIVKEY: {
									secret: {
										name: "ssh"
										key:  "private"
									}
								}
								DBSYNC_SSH_PUBKEY: {
									secret: {
										name: "ssh"
										key:  "public"
									}
								}
								DBSYNC_SSH_HOST_KEY: {
									secret: {
										name: "ssh"
										key:  "host"
									}
								}
								SSH_SNAPSHOT_TOOL_DESTINATION: {
									secret: {
										name: "ssh-address"
										key:  "address"
									}
								}
								IDEASCALE_API_URL: {
									secret: {
										name: "external"
										key:  "ideascale_url"
									}
								}
								IDEASCALE_API_TOKEN: {
									secret: {
										name: "external"
										key:  "ideascale_token"
									}
								}
								SNAPSHOT_TOOL_SSH: value:            string | *"true"
								SSH_SNAPSHOT_TOOL_PATH: value:       string | *"/home/snapshot/.local/bin/snapshot_tool"
								SSH_SNAPSHOT_TOOL_OUTPUT_DIR: value: string | *"/home/snapshot/dev_snapshot_tool_out"
								GVC_API_URL: value:                  string | *"unused"
								IS_NODE_RELOADABLE: value:           string | *"true"
								VOTING_HOST: value:                  string | *"unused"
								VOTING_PORT: value:                  string | *"8080"
								VOTING_LOG_LEVEL: value:             string | *"debug"
								JORM_PATH: value:                    string | *"jormungandr"
								JCLI_PATH: value:                    string | *"jcli"
								IDEASCALE_CONFIG_PATH: value:        string | *"/configs/ideascale.json"
								IDEASCALE_CAMPAIGN_GROUP: value:     string | *"66"
								IDEASCALE_STAGE_ID: value:           string | *"4385"
								IDEASCALE_LOG_LEVEL: value:          string | *"debug"
								IDEASCALE_LOG_FORMAT: value:         string | *"text"
								SNAPSHOT_INTERVAL_SECONDS: value:    string | *"1800"
								SNAPSHOT_CONFIG_PATH: value:         string | *"/app/snapshot-importer-example-config.json"
								SNAPSHOT_OUTPUT_DIR: value:          string | *"/tmp/snapshot-output"
								SNAPSHOT_NETWORK_IDS: value:         string | *"testnet mainnet"
								SNAPSHOT_LOG_LEVEL: value:           string | *"debug"
								SNAPSHOT_LOG_FORMAT: value:          string | *"text"
							}
							mounts: {
								ideascale: {
									ref: config: name: "ideascale"
									path:    "/configs/ideascale.json"
									subPath: "ideascale.json"
								}
								snapshot: {
									ref: volume: name: "snapshot"
									path:     "/tmp/snapshot-output"
									readOnly: false
								}
							}
							ports: {
								http: port: 8080
							}
						}
					}

					configs: ideascale: data: "ideascale.json": json.Marshal({
						proposals: {
							extra_field_mappings: {
								auto_translated:                "auto_translated"
								brief:                          "challenge_brief"
								budget_breakdown:               "please_provide_a_detailed_budget_breakdown"
								challenges_or_risks:            "what_main_challenges_or_risks_do_you_foresee_to_deliver_this_project_successfully_"
								full_solution:                  "please_describe_your_proposed_solution"
								goal:                           "how_does_success_look_like_"
								how_solution_address_challenge: "please_describe_how_your_proposed_solution_will_address_the_challenge_"
								importance:                     "importance"
								metrics:                        "key_metrics_to_measure"
								new_proposal:                   "is_this_proposal_is_a_continuation_of_a_previously_funded_project_in_catalyst__or_an_entirely_new_o"
								progress_metrics:               "what_will_you_measure_to_track_your_project_s_progress__and_how_will_you_measure_it_"
								relevant_link_1:                "relevant_link_1"
								relevant_link_2:                "website__github_repository__or_any_other_relevant_link__"
								relevant_link_3:                "relevant_link_3"
								return_in_a_later_round:        "if_you_are_funded__will_you_return_to_catalyst_in_a_later_round_for_further_funding__please_explain"
								sdg_rating:                     "sdg_rating"
								solution:                       "problem_solution"
								team_details:                   "please_provide_details_of_the_people_who_will_work_on_the_project_"
								timeline_and_key_milestones:    "please_provide_a_detailed_plan__a_timeline__and_key_milestones_for_delivering_your_proposal_"
							}
							field_mappings: {
								funds: [
									"requested_funds",
									"requested_funds_in_ada",
									"requested_funds_coti",
								]
								proposer_relevant_experience: "relevant_experience"
								proposer_url: [
									"relevant_link_1",
									"website__github_repository__or_any_other_relevant_link__",
									"relevant_link_3",
								]
								public_key: "ada_payment_address__"
							}
						}
						proposals_scores_csv: {
							id_field:    "proposal_id"
							score_field: "Rating"
						}
					})

					secrets: {
						external: {
							ref: "voting-node/external"
						}
						"eventdb-url": {
							ref: "db/eventdb"
							template: url: "postgres://{{ .username }}:{{ .password }}@{{ .host }}:{{ .port }}/eventdb"
						}
						generated: {
							ref: "voting-node/generated"
						}
						mainnet: {
							ref:    "db/dbsync-mainnet"
							global: true
						}
						"mainnet-url": {
							ref:    "db/dbsync-mainnet"
							global: true
							template: url: "postgres://{{ .username }}:{{ .password }}@{{ .host }}:{{ .port }}/{{ .dbInstanceIdentifier }}"
						}
						preprod: {
							ref:    "db/dbsync-preprod"
							global: true
						}
						"preprod-url": {
							ref:    "db/dbsync-preprod"
							global: true
							template: url: "postgres://{{ .username }}:{{ .password }}@{{ .host }}:{{ .port }}/{{ .dbInstanceIdentifier }}"
						}
						ssh: {
							ref:    "db/dbsync-ssh"
							global: true
						}
						"ssh-address": {
							ref:    "db/dbsync-ssh"
							global: true
							template: address: "{{ .user }}@{{ .host }}"
						}
					}

					service: {}

					volumes: snapshot: {
						size: "1Gi"
					}
				}
			}
		}
	}

	release: {
		docker: {
			on: {
				//merge: {}
				//tag: {}
				always: {}
			}
			config: {
				tag: _ @forge(name="GIT_HASH_OR_TAG")
			}
		}
	}
}
