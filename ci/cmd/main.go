package main

import (
	"encoding/json"
	"fmt"
	"os"
	"text/template"
	"time"

	"github.com/alecthomas/kong"
	"github.com/input-output-hk/catalyst-core/ci/pkg"
	"github.com/input-output-hk/catalyst-core/ci/pkg/executors"
	"github.com/input-output-hk/catalyst-core/ci/pkg/git_clients"
	"github.com/input-output-hk/catalyst-core/ci/pkg/scanners"
	"github.com/input-output-hk/catalyst-core/ci/pkg/util"
	"github.com/spf13/afero"
)

// TagTemplate is a template for generating image tags.
type TagTemplate struct {
	Hash      string
	Timestamp string
	Version   string
}

var cli struct {
	Tags tagsCmd `cmd:"" help:"Generate image tags with the current git context."`
	Scan scanCmd `cmd:"" help:"Scan for Earthfiles."`
}

type tagsCmd struct {
	TemplateString string `arg:"" help:"template for generating image tags" default:"{{ .Hash }}"`
}

func (c *tagsCmd) Run() error {
	executor := executors.NewLocalExecutor("git")
	client := git_clients.NewExternalGitClient(executor)

	// Collect the highest version from the git tags
	tags, err := client.Tags()
	if err != nil {
		return err
	}
	highest := util.GetHighestVersion(tags)

	// Get the current git commit hash
	hash, err := executor.Run("rev-parse", "HEAD")
	if err != nil {
		return err
	}

	// Get the current timestamp
	timestamp := time.Now().Format("20060102150405")

	// Generate the tag
	tmpl, err := template.New("tag").Parse(c.TemplateString)
	if err != nil {
		return err
	}

	data := TagTemplate{
		Hash:      hash,
		Timestamp: timestamp,
		Version:   highest,
	}
	err = tmpl.Execute(os.Stdout, data)
	if err != nil {
		return err
	}

	return nil
}

type scanCmd struct {
	Paths      []string `arg:"" help:"paths to scan for Earthfiles" type:"path"`
	Target     string   `short:"t" help:"filter by Earthfiles that include this target" default:""`
	JsonOutput bool     `short:"j" long:"json" help:"Output in JSON format"`
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

	if c.JsonOutput {
		jsonFiles, err := json.Marshal(files)
		if err != nil {
			return err
		}
		fmt.Println(string(jsonFiles))
	} else {
		for _, file := range files {
			fmt.Println(file.Path)
		}
	}

	return nil
}

func main() {
	ctx := kong.Parse(&cli)
	err := ctx.Run()
	ctx.FatalIfErrorf(err)
	os.Exit(0)
}
