version: "1.0.0"
project: {
	name: "voting-node"
    release: {
		docker: {
			on: {
				tag: {}
			}
			config: {
				tag: _ @forge(name="GIT_HASH_OR_TAG")
			}
		}
	}
}