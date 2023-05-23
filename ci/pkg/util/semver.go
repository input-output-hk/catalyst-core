package util

import (
	"sort"

	"github.com/Masterminds/semver/v3"
)

func GetHighestVersion(versions []string) string {
	var parsedVersions []*semver.Version
	for _, version := range versions {
		parsedVersion, err := semver.NewVersion(version)

		// We ignore invalid versions
		if err == nil {
			parsedVersions = append(parsedVersions, parsedVersion)
		}
	}

	// If there are no valid versions, we return v0.0.0
	if len(parsedVersions) == 0 {
		return "v0.0.0"
	}

	// We sort the versions in reverse order to get the highest version
	sort.Sort(sort.Reverse(semver.Collection(parsedVersions)))

	return parsedVersions[0].Original()
}
