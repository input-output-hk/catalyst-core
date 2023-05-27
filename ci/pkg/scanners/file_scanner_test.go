package scanners_test

import (
	"errors"

	"github.com/earthly/earthly/ast/spec"
	"github.com/input-output-hk/catalyst-core/ci/pkg"
	"github.com/input-output-hk/catalyst-core/ci/pkg/scanners"
	"github.com/spf13/afero"

	. "github.com/onsi/ginkgo/v2"
	. "github.com/onsi/gomega"
)

var _ = Describe("FileScanner", func() {
	var (
		fs     afero.Fs
		parser pkg.EarthfileParser
	)

	BeforeEach(func() {
		fs = afero.NewMemMapFs()
	})

	Describe("Scan", func() {
		BeforeEach(func() {
			afero.WriteFile(fs, "/test/Earthfile", []byte("target:"), 0644)
			afero.WriteFile(fs, "/test/pkg/Earthfile", []byte("target:"), 0644)
			parser = &mockParser{
				earthfile: pkg.Earthfile{},
				err:       nil,
			}
		})

		It("should return Earthfiles", func() {
			fScanner := scanners.NewFileScanner([]string{"/test"}, parser, fs)
			earthfiles, err := fScanner.Scan()
			Expect(err).NotTo(HaveOccurred())
			Expect(len(earthfiles)).To(Equal(2))
			Expect(earthfiles[0].Path).To(Equal("/test/Earthfile"))
			Expect(earthfiles[1].Path).To(Equal("/test/pkg/Earthfile"))
		})

		Context("when the parser fails", func() {
			BeforeEach(func() {
				parser = &mockParser{
					earthfile: pkg.Earthfile{},
					err:       errors.New("executor error"),
				}
			})

			It("should return an error", func() {
				fScanner := scanners.NewFileScanner([]string{"/test"}, parser, fs)
				_, err := fScanner.ScanForTarget("docker")
				Expect(err).To(MatchError("executor error"))
			})
		})
	})

	Describe("ScanForTarget", func() {
		BeforeEach(func() {
			afero.WriteFile(fs, "/test/Earthfile", []byte("docker"), 0644)
			parser = &mockParser{
				earthfile: pkg.Earthfile{
					Targets: []spec.Target{
						{
							Name: "docker",
						},
					},
				},
			}
		})

		It("should return Earthfiles with docker target", func() {
			fScanner := scanners.NewFileScanner([]string{"/test"}, parser, fs)
			earthfiles, err := fScanner.ScanForTarget("docker")
			Expect(err).NotTo(HaveOccurred())
			Expect(len(earthfiles)).To(Equal(1))
			Expect(earthfiles[0].Path).To(Equal("/test/Earthfile"))
		})

		Context("when the Earthfile does not contain docker target", func() {
			BeforeEach(func() {
				afero.WriteFile(fs, "/test/Earthfile", []byte("other"), 0644)
				parser = &mockParser{
					earthfile: pkg.Earthfile{
						Targets: []spec.Target{
							{
								Name: "other",
							},
						},
					},
				}
			})

			It("should return an empty slice", func() {
				fScanner := scanners.NewFileScanner([]string{"/test"}, parser, fs)
				earthfiles, err := fScanner.ScanForTarget("docker")
				Expect(err).NotTo(HaveOccurred())
				Expect(len(earthfiles)).To(Equal(0))
			})
		})
	})
})
