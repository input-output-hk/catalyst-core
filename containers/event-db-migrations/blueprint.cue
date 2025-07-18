project: {
	name: "event-db-migrations"
	deployment: {
		on: {
			merge: {}
			tag: {}
		}

		bundle:
		{
			env:  string | *"dev"
			_env: env
			modules: main: {
				name:    "app"
				version: "0.11.1"
				values: {
					jobs: migration: containers: main: {
						image: {
							name: _ @forge(name="CONTAINER_IMAGE")
							tag:  _ @forge(name="GIT_HASH_OR_TAG")
						}
						env: {
							DB_HOST: {
								secret: {
									name: "db"
									key:  "host"
								}
							}
							DB_NAME: {
								value: "eventdb"
							}
							DB_PORT: {
								secret: {
									name: "db"
									key:  "port"
								}
							}
							DB_ROOT_NAME: {
								value: "postgres"
							}
							DB_SUPERUSER: {
								secret: {
									name: "root"
									key:  "username"
								}
							}
							DB_SUPERUSER_PASSWORD: {
								secret: {
									name: "root"
									key:  "password"
								}
							}
							DB_USER: {
								secret: {
									name: "db"
									key:  "username"
								}
							}
							DB_USER_PASSWORD: {
								secret: {
									name: "db"
									key:  "password"
								}
							}
							INIT_AND_DROP_DB: {
								value: string | *"true"
							}
							STAGE: {
								value: string | *_env
							}
						}
						mounts: state: {
							ref: volume: name: "state"
							path:     "/eventdb/tmp"
							readOnly: false
						}
						resources: requests: {
							cpu:    "100m"
							memory: "128Mi"
						}
					}

					secrets: {
						db: {
							ref: "db/eventdb"
						}
						root: {
							ref: "db/root_account"
						}
					}

					volumes: state: {
						size: "1Mi"
					}
				}
			}
		}
	}

	release: {
		docker: {
			on: {
				merge: {}
				tag: {}
			}
			config: {
				tag: _ @forge(name="GIT_HASH_OR_TAG")
			}
		}
	}
}
