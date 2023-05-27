package scanners_test

import (
	"testing"

	"github.com/input-output-hk/catalyst-core/ci/pkg"
	. "github.com/onsi/ginkgo/v2"
	. "github.com/onsi/gomega"
)

type mockParser struct {
	earthfile pkg.Earthfile
	err       error
}

func (m *mockParser) Parse(path string) (pkg.Earthfile, error) {
	m.earthfile.Path = path
	return m.earthfile, m.err
}

func TestFileScanner(t *testing.T) {
	RegisterFailHandler(Fail)
	RunSpecs(t, "FileScanner Suite")
}
