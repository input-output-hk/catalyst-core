#!/usr/bin/env bash

set -o errexit -o pipefail -o noclobber -o nounset

ECR_REGISTRY="432820653916.dkr.ecr.eu-central-1.amazonaws.com"

echo "Clearing existing login sessions..."
docker logout "$ECR_REGISTRY"

echo "Logging into ECR registry..."
aws ecr get-login-password --region eu-central-1 | docker login --username AWS --password-stdin "$ECR_REGISTRY"
