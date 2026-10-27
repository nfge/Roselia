pub type ThreadId = u64;

pub struct Thread {
    pub id: ThreadId,
    pub context: ThreadContext,
    pub kernel_stack: KernelThreadStack,
    pub state: ThreadState
}

#[derive(Debug,Clone,Copy,PartialEq, Eq)]
pub enum ThreadState {
    Ready,
    Running,
    Sleeping,
    Stopped,
    Terminated
}

pub struct KernelThreadStack {
    pub ptr: u64,
    pub len: usize,
    pub top: u64
}

#[repr(C)]
#[derive(Default,Debug)]
pub struct ThreadContext {
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub r11: u64,
    pub r10: u64,
    pub r9: u64,
    pub r8: u64,

    pub rdi: u64,
    pub rsi: u64,
    pub rbp: u64,
    pub rdx: u64,
    pub rcx: u64,
    pub rbx: u64,
    pub rax: u64,

    pub rip: u64,
    pub cs: u64,
    pub rflags: u64
}