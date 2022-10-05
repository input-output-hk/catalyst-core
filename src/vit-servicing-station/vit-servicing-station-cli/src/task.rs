pub trait ExecTask {
    type ResultValue;
    type Error;
    fn exec(&self) -> Result<<Self as ExecTask>::ResultValue, <Self as ExecTask>::Error>;
}
