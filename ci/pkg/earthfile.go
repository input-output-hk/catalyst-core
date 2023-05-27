package pkg

import (
	"fmt"
	"regexp"
	"strings"

	"github.com/earthly/earthly/ast/spec"
)

// Earthfile represents a parsed Earthfile.
type Earthfile struct {
	BaseRecipe     spec.Block           `json:"baseRecipe"`
	Path           string               `json:"path,omitempty"`
	SourceLocation *spec.SourceLocation `json:"sourceLocation,omitempty"`
	Targets        []spec.Target        `json:"targets,omitempty"`
	UserCommands   []spec.UserCommand   `json:"userCommands,omitempty"`
	Version        *spec.Version        `json:"version,omitempty"`
}

func (e Earthfile) GetImages(target string) ([]string, error) {
	commands, err := e.GetCommands(target, "SAVE IMAGE")
	if err != nil {
		return nil, err
	}

	if len(commands) == 0 {
		return nil, nil
	}

	var images []string
	for _, command := range commands {
		for _, arg := range command.Args {
			if !strings.HasPrefix(arg, "--") {
				// Remove any variables from the string
				re := regexp.MustCompile(`\$\{[^}]+\}`)
				processedStr := re.ReplaceAllString(arg, "")

				// Remove any tags from the string
				imageName := strings.Split(processedStr, ":")[0]

				images = append(images, imageName)
			}
		}
	}

	return images, nil
}

func (e Earthfile) GetCommands(target string, command string) ([]*spec.Command, error) {
	var commands []*spec.Command
	t, err := e.GetTarget(target)
	if err != nil {
		return nil, err
	}

	for _, statement := range t.Recipe {
		if statement.Command != nil && statement.Command.Name == command {
			commands = append(commands, statement.Command)
		}
	}

	return commands, nil
}

func (e Earthfile) GetTarget(target string) (*spec.Target, error) {
	for _, t := range e.Targets {
		if t.Name == target {
			return &t, nil
		}
	}

	return nil, fmt.Errorf("target %s not found in %s", target, e.Path)
}

// EarthfileParser is an interface that can parse Earthfiles.
type EarthfileParser interface {
	Parse(path string) (Earthfile, error)
}

// EarthfileScanner is an interface that can scan for Earthfiles.
type EarthfileScanner interface {
	// Scan returns a list of Earthfiles.
	Scan() ([]Earthfile, error)

	// ScanForTarget returns a list of Earthfiles that contain the given target.
	ScanForTarget(target string) ([]Earthfile, error)
}
