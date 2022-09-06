use bytes::Bytes;
use mini_redis::client;
use tokio::sync::{mpsc, oneshot};

type Responder<T> = oneshot::Sender<mini_redis::Result<T>>;

#[derive(Debug)]
enum Command {
    Get {
        key: String,
        res: Responder<Option<Bytes>>,
    },
    Set {
        key: String,
        val: Bytes,
        res: Responder<()>,
    },
}

#[tokio::main]
async fn main() {
    // we open a channel which `client` tasks will use to pass messages to `manager` task
    // the channel is multiple producer - multiple consumer because the manager will serve
    // multiple `client` tasks
    // the channel closes when senders have been dropped or have gone out of scope
    let (tx, mut rx) = mpsc::channel(32);
    
    // to get more senders simply clone the sender
    let tx2 = tx.clone();

    // this is the manager task, it receives messages from async tasks and acts accordingly
    let manager = tokio::spawn(async move {
        // Establish a connection to the server
        let mut client = client::connect("127.0.0.1:6379").await.unwrap();


        while let Some(cmd) = rx.recv().await {
            // we match commands sent to the `manager` task to appropriate actions
            // towards the redis server 
            match cmd {
                Command::Get { key, res } => {
                    let resp = client.get(&key).await;

                    // we utilize the Responder struct in our Command struct to respond to
                    // the `client` task that invoked our `manager` task
                    let _ = res.send(resp);
                }
                Command::Set { key, val, res } => {
                    let resp = client.set(&key, val).await;

                    let _ = res.send(resp);
                }
            }
        }
    });

    // Spawn two tasks, one gets a key, the other sets a key
    let t1 = tokio::spawn(async move {
        let (res_tx, res_rx) = oneshot::channel();

        let cmd = Command::Get {
            key: "hello".to_string(),
            res: res_tx,
        };

        tx.send(cmd).await.unwrap();

        let res = res_rx.await;
        println!("GOT = {:?}", res);
    });

    let t2 = tokio::spawn(async move {
        let (res_tx, res_rx) = oneshot::channel();

        let cmd = Command::Set {
            key: "foo".to_string(),
            val: "bar".into(),
            res: res_tx,
        };

        tx2.send(cmd).await.unwrap();

        let res = res_rx.await;
        println!("GOT = {:?}", res);
    });

    t1.await.unwrap();
    t2.await.unwrap();
    manager.await.unwrap();
}
