use std::ptr;

const MAP_RW: libc::c_int = libc::PROT_READ | libc::PROT_WRITE;
const MAP_RX: libc::c_int = libc::PROT_READ | libc::PROT_EXEC;
const MAP_RWX: libc::c_int = libc::PROT_READ | libc::PROT_WRITE | libc::PROT_EXEC;

#[derive(Debug)]
pub enum MmapError {
    MmapFailed,
    MprotectFailed,
    SplitFailedNotEnoughSpace,
}

impl std::fmt::Display for MmapError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            MmapError::MmapFailed => write!(f, "mmap failed"),
            MmapError::MprotectFailed => write!(f, "mprotect failed"),
            MmapError::SplitFailedNotEnoughSpace => write!(f, "split failed: not enough space"),
        }
    }
}

/// A wrapper around a memory-mapped buffer.
///
/// The buffer is allocated with `mmap`.
#[derive(Debug)]
pub struct MmapBuf {
    ptr: *mut u8,
    size: usize,
    protect: libc::c_int,
}

impl MmapBuf {
    pub fn page_size() -> usize {
        unsafe { libc::sysconf(libc::_SC_PAGESIZE) as usize }
    }

    pub fn page_aligned_size(size: usize) -> usize {
        let page_size = Self::page_size();
        (size + page_size - 1) & !(page_size - 1)
    }

    pub fn new(size: usize) -> Result<MmapBuf, MmapError> {
        let size = Self::page_aligned_size(size);

        let ptr = unsafe {
            libc::mmap(
                ptr::null_mut(),
                size,
                MAP_RW,
                libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
                -1,
                0,
            )
        };

        if ptr == libc::MAP_FAILED {
            return Err(MmapError::MmapFailed);
        }

        Ok(MmapBuf {
            ptr: ptr as *mut u8,
            size,
            protect: MAP_RW,
        })
    }

    pub fn split_page_start(&mut self) -> Result<MmapBuf, MmapError> {
        if self.size < 2 * MmapBuf::page_size() {
            return Err(MmapError::SplitFailedNotEnoughSpace);
        }

        let ptr1 = self.ptr;
        let size1 = MmapBuf::page_size();

        // Update current MmapBuf
        self.ptr = unsafe { self.ptr.add(size1) };
        self.size = MmapBuf::page_aligned_size(self.size - size1);

        Ok(MmapBuf {
            ptr: ptr1,
            size: size1,
            protect: self.protect,
        })
    }

    pub fn split_page_end(&mut self) -> Result<MmapBuf, MmapError> {
        if self.size < 2 * MmapBuf::page_size() {
            return Err(MmapError::SplitFailedNotEnoughSpace);
        }

        self.size = MmapBuf::page_aligned_size(self.size - MmapBuf::page_size());

        let ptr2 = unsafe { self.ptr.add(self.size) };
        let size2 = MmapBuf::page_size();

        Ok(MmapBuf {
            ptr: ptr2,
            size: size2,
            protect: self.protect,
        })
    }

    pub fn split_end(&mut self, size: usize) -> Result<MmapBuf, MmapError> {
        if self.size < size {
            return Err(MmapError::SplitFailedNotEnoughSpace);
        }

        let ptr = unsafe { self.ptr.add(self.size - size) };
        self.size -= size;

        Ok(MmapBuf {
            ptr,
            size,
            protect: self.protect,
        })
    }

    pub fn ptr(&self) -> *const u8 {
        self.ptr
    }

    pub fn protect_no_access(&mut self) -> Result<(), MmapError> {
        self.protect(libc::PROT_NONE)
    }

    pub fn protect_rx(&mut self) -> Result<(), MmapError> {
        self.protect(MAP_RX)
    }

    pub fn protect_rwx(&mut self) -> Result<(), MmapError> {
        self.protect(MAP_RWX)
    }

    pub fn protect_rw(&mut self) -> Result<(), MmapError> {
        self.protect(MAP_RW)
    }

    pub fn protect(&mut self, protect: libc::c_int) -> Result<(), MmapError> {
        let ret = unsafe { libc::mprotect(self.ptr as *mut libc::c_void, self.size, protect) };

        if ret != 0 {
            return Err(MmapError::MprotectFailed);
        }

        self.protect = protect;
        Ok(())
    }

    pub fn is_executable(&self) -> bool {
        self.protect & libc::PROT_EXEC != 0
    }
}

impl Drop for MmapBuf {
    fn drop(&mut self) {
        unsafe {
            if libc::munmap(self.ptr as *mut libc::c_void, self.size) != 0 {
                panic!("munmap failed");
            }
        }
    }
}

#[derive(Debug)]
pub struct GuardedMmap {
    _guard_before: MmapBuf,
    mmap: MmapBuf,
    _guard_after: MmapBuf,
    map_name: String,
}

impl GuardedMmap {
    pub fn new(size: usize, map_name: String) -> Result<GuardedMmap, MmapError> {
        let size = MmapBuf::page_aligned_size(size) + 2 * MmapBuf::page_size();
        let mut mmap_buf = MmapBuf::new(size)?;

        let mut guard_before = mmap_buf.split_page_start()?;
        let mut guard_after = mmap_buf.split_page_end()?;

        guard_before.protect_no_access()?;
        guard_after.protect_no_access()?;

        Ok(GuardedMmap {
            _guard_before: guard_before,
            mmap: mmap_buf,
            _guard_after: guard_after,
            map_name,
        })
    }

    pub fn name(&self) -> &str {
        &self.map_name
    }

    pub fn ptr(&self) -> *const u8 {
        self.mmap.ptr()
    }

    pub fn protect_rx(&mut self) -> Result<(), MmapError> {
        self.mmap.protect_rx()
    }

    pub fn protect_rw(&mut self) -> Result<(), MmapError> {
        self.mmap.protect_rw()
    }

    pub fn is_executable(&self) -> bool {
        self.mmap.is_executable()
    }
}
