package executors

import "os/exec"

type LocalExecutor struct {
	Path string
}

func (l LocalExecutor) Run(args ...string) (string, error) {
	cmd := exec.Command(l.Path, args...)
	out, err := cmd.Output()
	return string(out), err
}

func NewLocalExecutor(path string) LocalExecutor {
	return LocalExecutor{
		Path: path,
	}
}
