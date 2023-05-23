package scanners_test

import (
	"errors"

	"github.com/input-output-hk/catalyst-core/ci/pkg"
	"github.com/input-output-hk/catalyst-core/ci/pkg/scanners"
	"github.com/spf13/afero"

	. "github.com/onsi/ginkgo/v2"
	. "github.com/onsi/gomega"
)

var _ = Describe("FileScanner", func() {
	var (
		fs afero.Fs
		ex pkg.Executor
	)

	BeforeEach(func() {
		fs = afero.NewMemMapFs()
	})

	Describe("Scan", func() {
		BeforeEach(func() {
			afero.WriteFile(fs, "/test/Earthfile", []byte("target:"), 0644)
			afero.WriteFile(fs, "/test/pkg/Earthfile", []byte("target:"), 0644)
			ex = &mockExecutor{
				commandOutput: "",
				err:           nil,
			}
		})

		It("should return Earthfiles", func() {
			fScanner := scanners.NewFileScanner([]string{"/test"}, ex, fs)
			earthfiles, err := fScanner.Scan()
			Expect(err).NotTo(HaveOccurred())
			Expect(len(earthfiles)).To(Equal(2))
			Expect(earthfiles[0].Path).To(Equal("/test"))
			Expect(earthfiles[1].Path).To(Equal("/test/pkg"))
		})

		Context("when the executor fails", func() {
			BeforeEach(func() {
				ex = &mockExecutor{
					commandOutput: "",
					err:           errors.New("executor error"),
				}
			})

			It("should return an error", func() {
				fScanner := scanners.NewFileScanner([]string{"/test"}, ex, fs)
				_, err := fScanner.ScanForTarget("docker")
				Expect(err).To(MatchError("executor error"))
			})
		})
	})

	Describe("ScanForTarget", func() {
		BeforeEach(func() {
			afero.WriteFile(fs, "/test/Earthfile", []byte("+docker"), 0644)
			ex = &mockExecutor{
				commandOutput: "+docker\n",
				err:           nil,
			}
		})

		It("should return Earthfiles with +docker target", func() {
			fScanner := scanners.NewFileScanner([]string{"/test"}, ex, fs)
			earthfiles, err := fScanner.ScanForTarget("+docker")
			Expect(err).NotTo(HaveOccurred())
			Expect(len(earthfiles)).To(Equal(1))
			Expect(earthfiles[0].Path).To(Equal("/test"))
		})

		Context("when the Earthfile does not contain +docker target", func() {
			BeforeEach(func() {
				afero.WriteFile(fs, "/test/Earthfile", []byte("+other"), 0644)
				ex = &mockExecutor{
					commandOutput: "+other\n",
					err:           nil,
				}
			})

			It("should return an empty slice", func() {
				fScanner := scanners.NewFileScanner([]string{"/test"}, ex, fs)
				earthfiles, err := fScanner.ScanForTarget("docker")
				Expect(err).NotTo(HaveOccurred())
				Expect(len(earthfiles)).To(Equal(0))
			})
		})
	})
})
