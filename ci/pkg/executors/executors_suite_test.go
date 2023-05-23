package executors_test

import (
	"testing"

	. "github.com/onsi/ginkgo/v2"
	. "github.com/onsi/gomega"
)

func TestEarthlyExecutor(t *testing.T) {
	RegisterFailHandler(Fail)
	RunSpecs(t, "EarthlyExecutor Suite")
}
