package scanners_test

import (
	"testing"

	. "github.com/onsi/ginkgo/v2"
	. "github.com/onsi/gomega"
)

type mockExecutor struct {
	commandOutput string
	err           error
}

func (m *mockExecutor) Run(args ...string) (string, error) {
	return m.commandOutput, m.err
}

func TestFileScanner(t *testing.T) {
	RegisterFailHandler(Fail)
	RunSpecs(t, "FileScanner Suite")
}
