use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use time::Instant;

const THREAD_COUNT: usize = 12;
const WORK_COUNT: usize = 2000;

#[test]
fn test_chunk_threading() {
    // let join_handles: Arc<Mutex<Vec<JoinHandle<()>>>> = Arc::new(Mutex::new(Vec::new()));
    let completed_tasks = Arc::new(AtomicUsize::new(0));

    let pool = threadpool::Builder::new().num_threads(THREAD_COUNT).build();
    // let pool = rayon::ThreadPoolBuilder::new()
    //     .num_threads(THREAD_COUNT)
    //     .spawn_handler(|builder| {
    //         let handle = thread::Builder::new().spawn(|| builder.run()).unwrap();
    //         join_handles.lock().unwrap().push(handle);
    //         Ok(())
    //     })
    //     .build()
    //     .unwrap();

    let start_time = Instant::now();
    let completed_tasks_clone = completed_tasks.clone();
    (0..WORK_COUNT)
        .collect::<Vec<_>>()
        .iter()
        .for_each(move |&index| {
            let completed_tasks_clone = completed_tasks_clone.clone();
            pool.execute(move || {
                let work = work(WORK_COUNT);
                if index % 100 == 0 {
                    println!("{}, {:?}", work, thread::current().id());
                }
                completed_tasks_clone.fetch_add(1, Relaxed);
            })
        });

    // let handles_clone = join_handles.clone();
    let completed_tasks_clone = completed_tasks.clone();
    std::thread::spawn(move || loop {
        // let mut finished_threads = 0;
        // for handle in handles_clone.lock().unwrap().iter() {
        //     if handle.is_finished() {
        //         finished_threads += 1;
        //     }
        // }
        if completed_tasks_clone.load(Relaxed) == WORK_COUNT {
            let elapsed_time = (Instant::now() - start_time).whole_microseconds();
            println!("Elapsed time (microseconds): {}", elapsed_time);
            break;
        }
    })
    .join()
    .unwrap();
}

fn work(n: usize) -> String {
    for _ in 0..n {
        let mut vec = vec![];
        for j in 0..n {
            vec.push(j);
        }
    }
    "Done".to_string()
}
