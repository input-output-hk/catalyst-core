use jortestkit::prelude::benchmark_consumption;
use jortestkit::prelude::ResourcesUsage;

#[test]
#[ignore]
pub fn benchmark_consumption_test() {
    let benchmark_consumption_monitor =
        benchmark_consumption("tallying_public_vote_with_10_000_votes")
            .target(ResourcesUsage::new(10, 200_000, 5_000_000))
            .for_process("Node", 48384)
            .start_async(std::time::Duration::from_secs(1));

    std::thread::sleep(std::time::Duration::from_secs(30));

    println!("{:?}", benchmark_consumption_monitor.stop().print());
}
