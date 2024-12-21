#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
extern crate alloc;

use core::{
    pin::Pin,
    future::Future,
    task::{Context, Waker, Poll, RawWaker, RawWakerVTable},
};
use alloc::{collections::VecDeque, boxed::Box};
use user_lib::get_time;

// Task 封装异步 Future 和任务的唯一 ID
pub struct Task {
    id: usize,
    future: Pin<Box<dyn Future<Output = ()>>>,
}

impl Task {
    // 对任务的 Future 进行轮询，尝试推进任务的状态
    // 如果任务未完成，返回 Poll::Pending；如果任务完成，返回 Poll::Ready
    fn poll(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        self.future.as_mut().poll(cx)
    }
}

// 异步运行时runtime
pub struct Runtime {
    ready_queue: VecDeque<Task>, // 准备就绪的任务队列
}

impl Runtime {
    // 创建运行时实例
    fn new() -> Self {
        Runtime {
            ready_queue: VecDeque::new(),
        }
    }

    // 运行时主循环，循环从就绪队列中取出任务进行轮询
    // 如果任务未完成，将其重新加入队列
    pub fn run(&mut self) {
        while let Some(mut task) = self.ready_queue.pop_front() {
            let waker = waker();                    // 创建一个空操作的 Waker
            let mut context = Context::from_waker(&waker);
            if let Poll::Pending = task.poll(&mut context) {
                self.ready_queue.push_back(task); // 如果任务未完成，将其重新加入队列
            }
        }
    }

    // 将异步任务封装成 Task 对象并加入就绪队列
    pub fn spawn(&mut self, future: impl Future<Output = ()> + Send + Sync + 'static) {
        let task = Task {
            id: self.ready_queue.len(),
            future: Box::pin(future), 
        };
        self.ready_queue.push_back(task);
    }
}

// 创建一个空操作的 Waker，用于任务上下文创建
// 该 Waker 不执行任何实际操作，目前runtime不支持外部唤醒
fn waker() -> Waker {
    unsafe fn no_op(_: *const ()) {}

    unsafe fn dummy_clone(_: *const ()) -> RawWaker {
        RawWaker::new(core::ptr::null(), &DUMMY_WAKER_VTABLE)
    }

    static DUMMY_WAKER_VTABLE: RawWakerVTable = RawWakerVTable::new(
        dummy_clone,
        no_op,
        no_op,
        no_op,
    );

    let raw_waker = RawWaker::new(core::ptr::null(), &DUMMY_WAKER_VTABLE);
    unsafe { Waker::from_raw(raw_waker) }
}

// 延迟
pub struct Delay {
    target_time: usize,
}

impl Delay {
    pub fn new(ms: usize) -> Self {
        Delay {
            target_time: get_time() as usize + ms,      // 通过syscall获取当前时间
        }
    }
}

impl Future for Delay {
    type Output = ();

    // 对延迟操作进行轮询，检查目标时间是否已到
    // 如果时间到，返回 Poll::Ready，否则返回 Poll::Pending
    fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
        if get_time() as usize >= self.target_time {
            Poll::Ready(())
        } else {
            Poll::Pending 
        }
    }
}

#[no_mangle]
pub fn main() -> i32 {
    let mut rt = Runtime::new();
    
    rt.spawn(multi_delay_task());
    
    rt.spawn(task_chain());
    
    rt.spawn(concurrent_task_1());
    rt.spawn(concurrent_task_2());
    rt.spawn(concurrent_task_3());
    
    rt.run();
    
    println!("All tasks completed!");
    0
}

async fn multi_delay_task() {
    println!("multi delay task started");
    Delay::new(100).await;
    println!("after first delay");
    Delay::new(300).await;
    println!("after second delay");
    Delay::new(400).await;
    println!("multi delay task completed");
}

async fn task_chain() {
    task_a().await; 
    task_b().await; 
    task_c().await; 
}


async fn task_a() {
    println!("task A started");
    Delay::new(300).await;
    println!("task A completed");
}


async fn task_b() {
    println!("task B started");
    Delay::new(400).await;
    println!("task B completed");
}


async fn task_c() {
    println!("task C started");
    Delay::new(500).await;
    println!("task C completed");
}


async fn concurrent_task_1() {
    println!("concurrent task 1 started");
    Delay::new(1000).await;
    println!("concurrent task 1 completed");
}


async fn concurrent_task_2() {
    println!("concurrent task 2 started");
    Delay::new(300).await;
    println!("concurrent task 2 completed");
}


async fn concurrent_task_3() {
    println!("concurrent task 3 started");
    Delay::new(600).await;
    println!("concurrent task 3 completed");
}

