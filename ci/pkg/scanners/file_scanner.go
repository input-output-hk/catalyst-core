package scanners

import (
	"os"
	"path/filepath"

	"github.com/input-output-hk/catalyst-core/ci/pkg"
	"github.com/spf13/afero"
)

// FileScanner is an interface that can scan for Earthfiles at the given paths.
type FileScanner struct {
	fs     afero.Fs
	parser pkg.EarthfileParser
	paths  []string
}

func (f *FileScanner) Scan() ([]pkg.Earthfile, error) {
	earthfiles, err := f.scan(func(e pkg.Earthfile) (bool, error) {
		return true, nil
	})
	if err != nil {
		return nil, err
	}

	return earthfiles, nil
}

func (f *FileScanner) ScanForTarget(target string) ([]pkg.Earthfile, error) {
	earthfiles, err := f.scan(func(e pkg.Earthfile) (bool, error) {
		for _, t := range e.Targets {
			if t.Name == target {
				return true, nil
			}
		}

		return false, nil
	})

	if err != nil {
		return nil, err
	}
	return earthfiles, nil
}

func (f *FileScanner) scan(filter func(pkg.Earthfile) (bool, error)) ([]pkg.Earthfile, error) {
	var earthfiles []pkg.Earthfile
	for _, path := range f.paths {
		if err := afero.Walk(f.fs, path, func(path string, info os.FileInfo, err error) error {
			// Filter to Earthfiles
			if filepath.Base(path) != "Earthfile" {
				return nil
			}

			earthfile, err := f.parser.Parse(path)
			if err != nil {
				return err
			}

			include, err := filter(earthfile)
			if err != nil {
				return err
			}
			if include {
				earthfiles = append(earthfiles, earthfile)
			}

			return nil
		}); err != nil {
			return nil, err
		}
	}

	return earthfiles, nil
}

// NewFileScanner creates a new FileScanner.
func NewFileScanner(paths []string, parser pkg.EarthfileParser, fs afero.Fs) *FileScanner {
	return &FileScanner{
		fs:     fs,
		parser: parser,
		paths:  paths,
	}
}
