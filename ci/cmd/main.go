package main

import (
	"fmt"
	"os"

	"github.com/alecthomas/kong"
	"github.com/input-output-hk/catalyst-core/ci/pkg"
	"github.com/input-output-hk/catalyst-core/ci/pkg/executors"
	"github.com/input-output-hk/catalyst-core/ci/pkg/scanners"
	"github.com/spf13/afero"
)

var cli struct {
	Tags tagsCmd `cmd:"" help:"Generate image tags with the current git context."`
	Scan scanCmd `cmd:"" help:"Scan for Earthfiles."`
}

type tagsCmd struct {
}

func (c *tagsCmd) Run() error {
	// TODO: Implement
	return nil
}

type scanCmd struct {
	Paths  []string `arg:"" help:"paths to scan for Earthfiles" type:"path"`
	Target string   `short:"t" help:"filter by Earthfiles that include this target" default:""`
}

func (c *scanCmd) Run() error {
	executor := executors.NewLocalExecutor("earthly")
	scanner := scanners.NewFileScanner(c.Paths, executor, afero.NewOsFs())

	var files []pkg.Earthfile
	var err error
	if c.Target != "" {
		files, err = scanner.ScanForTarget(c.Target)
	} else {
		files, err = scanner.Scan()
	}

	if err != nil {
		return err
	}

	for _, file := range files {
		fmt.Println(file.Path)
	}

	return nil
}

func main() {
	ctx := kong.Parse(&cli)
	err := ctx.Run()
	ctx.FatalIfErrorf(err)
	os.Exit(0)
}
