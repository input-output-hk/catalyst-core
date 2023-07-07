WIP

Integration tests are divided in four crates, and organized in modules. This is to prevent `rustc` to re-link the library crates with each of the integration tests (one for each *.rs file / test crate under the `tests/` folder). Performance implication: https://github.com/rust-lang/cargo/pull/5022#issuecomment-364691154  
This is also good for execution performance. Cargo will run all tests from a single binary in parallel, but binaries themselves are run sequentally.  

To run all tests in this folder:  
`cargo test --test '*'`  

To run only component tests:  
`cargo test --test component`  

To run only end to end tests:  
`cargo test --test end2end`  

To run only integration tests:  
`cargo test --test integration`  

To run only non functional tests:  
`cargo test --test non functional`  
