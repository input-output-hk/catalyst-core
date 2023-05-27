package parsers

import (
	"context"

	"github.com/earthly/earthly/ast"
	"github.com/earthly/earthly/ast/spec"
	"github.com/input-output-hk/catalyst-core/ci/pkg"
)

// EarthlyParser implements an EarthfileParser using the Earthly AST parser.
type EarthlyParser struct {
	AstParser AstParser
}

func (e EarthlyParser) Parse(path string) (pkg.Earthfile, error) {
	ef, err := e.AstParser.Parse(context.Background(), path, false)
	if err != nil {
		return pkg.Earthfile{}, err
	}

	return pkg.Earthfile{
		BaseRecipe:     ef.BaseRecipe,
		Path:           path,
		SourceLocation: ef.SourceLocation,
		Targets:        ef.Targets,
		UserCommands:   ef.UserCommands,
		Version:        ef.Version,
	}, nil
}

func NewEarthlyParser() EarthlyParser {
	return EarthlyParser{
		AstParser: EarthlyAstParser{},
	}
}

// AstParser wraps the Earthly AST parser.
type AstParser interface {
	Parse(ctx context.Context, path string, useCopy bool) (spec.Earthfile, error)
}

// EarthlyAstParser implements an AstParser using the Earthly AST parser.
type EarthlyAstParser struct{}

func (e EarthlyAstParser) Parse(ctx context.Context, path string, useCopy bool) (spec.Earthfile, error) {
	return ast.Parse(ctx, path, useCopy)
}
