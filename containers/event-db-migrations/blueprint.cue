version: "1.0.0"
project: {
	name: "event-db-migrations"
	release: {
		docker: {
			on: {
				always: {}
				//merge: {}
				//tag: {}
			}
			config: {
				tag: _ @forge(name="GIT_HASH_OR_TAG")
			}
		}
	}
}
