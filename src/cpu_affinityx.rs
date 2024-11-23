use libc::{cpu_set_t, sched_getaffinity, sched_setaffinity, CPU_ISSET, CPU_SET, CPU_ZERO};
use std::process;

/// set the cpu affinity for the current process to a specific core.
pub fn set_cpu_affinityx(core_id: usize) -> Result<(), String> {
    unsafe {
        let mut cpu_set: cpu_set_t = std::mem::zeroed();
        CPU_ZERO(&mut cpu_set);
        CPU_SET(core_id, &mut cpu_set);

        let pid = process::id() as libc::pid_t;
        let result = sched_setaffinity(pid, std::mem::size_of::<cpu_set_t>(), &cpu_set);

        if result == 0 {
            println!("successfully set cpu affinity to core {}", core_id);
            Ok(())
        } else {
            let errno = *libc::__errno_location();
            let err_msg = match errno {
                libc::EINVAL => "invalid core id".to_string(),
                libc::ESRCH => "process not found".to_string(),
                libc::EPERM => "permission denied".to_string(),
                _ => format!("unknown error: {}", errno),
            };
            Err(format!("failed to set cpu affinity: {}", err_msg))
        }
    }
}

/// get the current cpu affinity of the process.
pub fn get_cpu_affinityx() -> Result<Vec<usize>, String> {
    unsafe {
        let mut cpu_set: cpu_set_t = std::mem::zeroed();
        let pid = process::id() as libc::pid_t;
        let result = sched_getaffinity(pid, std::mem::size_of::<cpu_set_t>(), &mut cpu_set);

        if result == 0 {
            let mut cores = Vec::new();
            for i in 0..libc::CPU_SETSIZE as usize {
                if CPU_ISSET(i, &cpu_set) {
                    cores.push(i);
                }
            }
            Ok(cores)
        } else {
            Err("failed to get cpu affinity".to_string())
        }
    }
}
