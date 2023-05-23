package git_clients_test

import (
	"errors"

	"github.com/input-output-hk/catalyst-core/ci/pkg/git_clients"
	. "github.com/onsi/ginkgo/v2"
	. "github.com/onsi/gomega"
)

var _ = Describe("Tags", func() {
	It("should return a list of tags", func() {
		ex := &mockExecutor{
			commandOutput: "v1.0.0\nv1.1.0\nv1.2.0",
		}
		c := git_clients.NewExternalGitClient(ex)

		tags, err := c.Tags()
		Expect(err).To(BeNil())
		Expect(tags).To(Equal([]string{"v1.0.0", "v1.1.0", "v1.2.0"}))
	})

	It("should return an error if the executor returns an error", func() {
		ex := &mockExecutor{
			commandOutput: "",
			err:           errors.New("failed to run command"),
		}
		c := git_clients.NewExternalGitClient(ex)

		tags, err := c.Tags()
		Expect(err).NotTo(BeNil())
		Expect(tags).To(BeNil())
	})
})
