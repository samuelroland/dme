use std::process::Command;

pub fn command_new(name: &str) -> Command {
    #[allow(unused_mut)]
    let mut cmd = Command::new(name);
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        // Note: By default on Windows, running a process is going to open a black console that is very
        // annoying and completly useless. To avoid opening this console, we can set a flag to disable it.
        // https://stackoverflow.com/questions/68224966/using-stdprocesscommand-with-windows-subsystem-windows-causes-console-flas

        // List of all process creation flags: https://learn.microsoft.com/en-us/windows/win32/procthread/process-creation-flags
        const CREATE_NO_WINDOW: u32 = 0x08000000; // Or `134217728u32`
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd
}
