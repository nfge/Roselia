use core::mem::MaybeUninit;

pub type ThreadId = u64;

pub struct Thread {
    pub id: ThreadId,
    pub parent_id: Option<ThreadId>,
    pub context: ThreadContext,
    pub kernel_stack: KernelThreadStack,
    pub state: ThreadState, 
    pub exit_code: Option<i32>,
    pub reserved: bool
}

impl Thread {
    pub const fn new(id: ThreadId, ctx: ThreadContext, kernel_stack:KernelThreadStack, state:ThreadState) -> Self {
        Self {
            id: id,
            parent_id: None,
            context: ctx,
            kernel_stack: kernel_stack,
            state: state,
            exit_code: None,
            reserved: true
        }
    }
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

impl KernelThreadStack {
    pub const fn new(ptr: u64, len: usize, top: u64) -> Self {
        Self {
            ptr: ptr,
            len: len,
            top: top
        }
    }
}



#[repr(C)]
#[derive(Default,Debug, Clone, Copy)]
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
    pub rflags: u64,
    pub rsp: u64,
    pub ss: u64
}