use std::thread::JoinHandle;

/// Spawn a thread with a priority of the current thread + `priority` (lower value means higher priority).
/// Higher priority threads will always take precendence if they are ready, so setting a lower priority
/// (`priority > 0`) means it will run when the current thread is sleeping (basically run in the "background"),
/// and a higher priority (`priority < 0`) means it could pause the current thread until it yields.
pub fn spawn_thread<F, T>(priority: i32, f: F) -> JoinHandle<T>
where
    F: FnOnce() -> T,
    F: Send + 'static,
    T: Send + 'static,
{
    let (policy, mut param) = unsafe {
        let thread = libc::pthread_self();
        let mut policy: libc::c_int = 0;
        let mut priority: libc::sched_param = libc::sched_param { sched_priority: 0 };
        let res = libc::pthread_getschedparam(thread, &mut policy, &mut priority);
        if res < 0 {
            panic!("Couldn't get thread priority. Code {res}, Handle {thread}");
        }
        (policy, priority)
    };

    std::thread::spawn(move || unsafe {
        param.sched_priority += priority;

        let thread = libc::pthread_self();
        let res = libc::pthread_setschedparam(thread, policy, &param);
        if res < 0 {
            panic!("Couldn't set thread priority. Code {res}, Handle {thread}");
        }
        f()
    })
}
