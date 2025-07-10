project: {
	name: "cat-data-service"
	deployment: {
		on: {
			tag: {}
			merge: {}
		}
		bundle: {
			env: string | *"dev"
			modules: main: {
				name:    "app"
				version: "0.11.1"
				values: {
					deployment: containers: main: {
						image: {
							name: _ @forge(name="CONTAINER_IMAGE")
							tag:  _ @forge(name="GIT_HASH_OR_TAG")
						}

						env: {
							"DATABASE_URL": {
								secret: {
									name: "eventdb"
									key:  "url"
								}
							}
						}

						ports: {
							http: port: 3030
						}
					}

					dns: subdomain: "api"

					route: rules: [
						{
							matches: [
								{
									path: {
										type:  "PathPrefix"
										value: "/"
									}
								},
							]
							target: port: 80
						},
					]

					secrets: {
						eventdb: {
							ref: "db/eventdb"
							template: url: "postgres://{{ .username }}:{{ .password }}@{{ .host }}:{{ .port }}/eventdb"
						}
					}

					service: {
						ports: http: 80
					}
				}
			}
		}
	}

	release: {
		docker: {
			on: {
				tag: {}
				merge: {}
			}
			config: {
				tag: _ @forge(name="GIT_HASH_OR_TAG")
			}
		}
	}
}
