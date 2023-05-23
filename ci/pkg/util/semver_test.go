package util_test

import (
	. "github.com/onsi/ginkgo/v2"
	. "github.com/onsi/gomega"

	"github.com/input-output-hk/catalyst-core/ci/pkg/util"
)

var _ = Describe("GetHighestVersion", func() {
	It("should return the highest version", func() {
		versions := []string{"v1.0.0", "v1.1.0", "v1.2.0", "v2.0.0", "v2.1.0", "v2.1.1"}

		highestVersion := util.GetHighestVersion(versions)

		Expect(highestVersion).To(Equal("v2.1.1"))
	})

	It("should return v0.0.0 if there are no valid versions", func() {
		versions := []string{"invalid", "version", "strings"}

		highestVersion := util.GetHighestVersion(versions)

		Expect(highestVersion).To(Equal("v0.0.0"))
	})

	It("should ignore invalid versions", func() {
		versions := []string{"v1.0.0", "invalid", "v1.1.0", "version", "v1.2.0", "strings"}

		highestVersion := util.GetHighestVersion(versions)

		Expect(highestVersion).To(Equal("v1.2.0"))
	})
})
