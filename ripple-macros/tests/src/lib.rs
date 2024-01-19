#[cfg(test)]
pub mod tests {
    use ripple_proc_macros::timed;
    use ripple_sdk::tokio;
    use std::{thread, time::Duration};

    #[timed]
    pub fn stand_up_and_be_timed_no_args() {
        println!("asdfasdf");
    }

    #[timed]
    pub fn stand_up_and_be_timed_with_args(_input: String, _count: u32) {
        println!("asdfasdf");
        thread::sleep(Duration::new(1, 0));
    }
    #[timed]
    pub async fn async_stand_up_and_be_timed_no_args() {
        println!("asdfasdf,now with async");
        thread::sleep(Duration::new(1, 0));
    }
    #[test]
    pub fn test_timeded_no_args() {
        stand_up_and_be_timed_no_args();
        assert!(true);
    }

    #[test]
    pub fn test_timed_with_args() {
        stand_up_and_be_timed_with_args(String::from("foo"), 42);
        assert!(true);
    }
    #[tokio::test]
    pub async fn async_test_timed_no_args() {
        async_stand_up_and_be_timed_no_args().await;
        assert!(true);
    }
}
