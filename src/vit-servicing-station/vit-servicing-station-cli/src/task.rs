pub trait ExecTask {
    type ResultValue;
    fn exec(&self) -> std::io::Result<<Self as ExecTask>::ResultValue>;
}
