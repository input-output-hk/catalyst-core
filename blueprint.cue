global: {
	ci: {
		local: [
			"^check(-.*)?$",
			"^build(-.*)?$",
			"^package(-.*)?$",
			"^test(-.*)?$",
		]
		registries: [
			ci.providers.aws.ecr.registry,
		]
		providers: {
			aws: {
				region: "eu-central-1"
				ecr: registry: "332405224602.dkr.ecr.eu-central-1.amazonaws.com"
				role: "arn:aws:iam::332405224602:role/ci"
			}

			docker: credentials: {
				provider: "aws"
				path:     "global/ci/docker"
			}

			git: credentials: {
				provider: "aws"
				path:     "global/ci/deploy"
			}

			earthly: {
				credentials: {
					provider: "aws"
					path:     "global/ci/earthly"
				}
				org:       "Catalyst"
				satellite: "ci"
				version:   "0.8.15"
			}

			github: registry: "ghcr.io"

			kcl: {
				install: true
				version: "v0.11.0"
			}
		}
		secrets: [
			{
				name:     "GITHUB_TOKEN"
				optional: true
				provider: "env"
				path:     "GITHUB_TOKEN"
			},
		]
	}
	deployment: {
		registries: {
			containers: ci.providers.aws.ecr.registry
			modules:    ci.providers.aws.ecr.registry + "/catalyst-deployments"
		}
		repo: {
			url: "https://github.com/input-output-hk/catalyst-world"
			ref: "master"
		}
		root: "k8s"
	}
	repo: {
		defaultBranch: "main"
		name:          "input-output-hk/catalyst-core"
	}
}
