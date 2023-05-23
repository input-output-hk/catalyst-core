package pkg

// GitClient is an interface for interacting with git
type GitClient interface {
	// Tags returns a list of tags in the repository
	Tags() ([]string, error)
}
