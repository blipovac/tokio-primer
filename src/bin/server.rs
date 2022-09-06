use tokio::net::{TcpListener, TcpStream};
use mini_redis::{Connection, Frame};
use std::{sync::{Arc, Mutex}};
use bytes::Bytes;
use std::collections::HashMap;

// type keyword is used to create an alias for a long ass type like this
type Db = Arc<Mutex<HashMap<String, Bytes>>>;
type ShardedDb = Arc<Vec<Mutex<HashMap<String, Bytes>>>>;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    let mut db = new_sharded_db(1);

    loop {
        let (socket, _) = listener.accept().await.unwrap();

        let db = db.clone();

        // creates a task for the tokio scheduler returns a JoinHandle which
        // when awaited returns a Result<>

        // a task has a `static lifetime which means it must not contain
        // references to data owned outside the task
        // that's why we use `async move` to move ownership of the data
        // to the task
        tokio::spawn(async move {
            process(socket, db).await;
        });
    }
}

async fn process(socket: TcpStream, db: ShardedDb) {
    use mini_redis::Command::{self, Get, Set};

    let mut connection = Connection::new(socket);

    while let  Some(frame) = connection.read_frame().await.unwrap() {
        let response = match Command::from_frame(frame).unwrap() {
            Set(cmd) => {
                let mut shard = db[0].lock().unwrap();

                shard.insert(cmd.key().to_string(), cmd.value().clone());
                Frame::Simple("OK".to_string())
            }
            Get(cmd) => {
                let shard = db[0].lock().unwrap();

                if let Some(value) = shard.get(cmd.key()) {
                    Frame::Bulk(value.clone())
                } else {
                    Frame::Null
                }
            }
            cmd => panic!("unimplemented {:?}", cmd),
        };

        connection.write_frame(&response).await.unwrap();
    }
}

fn new_sharded_db(num_shards: usize) -> ShardedDb {
    // creates a vector of specified capacity, IMPORTANT: even though it has
    // capacity the len is 0
    let mut db = Vec::with_capacity(num_shards);

    for _ in 0..num_shards {
        db.push(Mutex::new(HashMap::new()));
    }

    Arc::new(db)
}