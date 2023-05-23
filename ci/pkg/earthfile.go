package pkg

import "strings"

// Earthfile represents an Earthfile.
type Earthfile struct {
	exec Executor
	Path string
}

// Targets returns a list of targets in the Earthfile.
func (e Earthfile) Targets() ([]string, error) {
	output, err := e.exec.Run("ls", e.Path)
	if err != nil {
		return nil, err
	}

	return strings.Split(output, "\n"), nil
}

// EarthfileScanner is an interface that can scan for Earthfiles.
type EarthfileScanner interface {
	// Scan returns a list of Earthfiles.
	Scan() ([]Earthfile, error)

	// ScanForTarget returns a list of Earthfiles that contain the given target.
	ScanForTarget(target string) ([]Earthfile, error)
}

// NewEarthfile returns a new Earthfile.
func NewEarthfile(path string, executor Executor) Earthfile {
	return Earthfile{
		exec: executor,
		Path: path,
	}
}
