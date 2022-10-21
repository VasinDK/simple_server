use simple_server;

#[test]
fn it_pool_sender() -> Result<(), String> {
    let pool = simple_server::ThreadPool::new(3);

    if let Some(_sender) = &pool.sender {
        Ok(())
    } else {
        Err(String::from("Not sender"))
    }
}