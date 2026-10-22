pub struct ChildGuard {
    pub child: tokio::process::Child,
    pub disarmed: bool,
}

impl Drop for ChildGuard {
    fn drop(&mut self) {
        if !self.disarmed {
            let _ = self.child.start_kill();
        }
    }
}
