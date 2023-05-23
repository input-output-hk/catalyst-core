package pkg

// Executor is an interface that can execute programs.
type Executor interface {
	// Run executes a program.
	Run(args ...string) (string, error)
}
