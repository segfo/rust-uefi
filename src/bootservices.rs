use core::ptr;
use core::mem;

use void::{NotYetDef, CVoid};
use base::{Event, Handle, Handles, MemoryType, MemoryDescriptor, Status};
use protocols;
use guid;
use table;

#[repr(C)]
pub enum LocateSearchType {
    AllHandles = 0,
    ByRegisterNotify = 1,
    ByProtocol = 2,
}

/// See http://wiki.phoenix.com/wiki/index.php/EFI_BOOT_SERVICES
#[repr(C)]
pub struct BootServices {
    header: table::TableHeader,
    raise_tpl: *const NotYetDef,
    restore_tpl: *const NotYetDef,
    allocate_pages: *const NotYetDef,
    free_pages: *const NotYetDef,
    get_memory_map: unsafe extern "win64" fn(memory_map_size: *mut usize, memory_map: *mut MemoryDescriptor, *mut usize, descriptor_size: *mut usize, descriptor_version: *mut u32) -> Status,
    allocate_pool: unsafe extern "win64" fn(pool_type: MemoryType, size: usize, out: &mut *mut CVoid) -> Status,
    free_pool: unsafe extern "win64" fn(*mut CVoid),
    create_event: *const NotYetDef,
    set_timer: *const NotYetDef,
    // typedef EFI_STATUS (EFIAPI *EFI_WAIT_FOR_EVENT) (IN UINTN NumberOfEvents, IN EFI_EVENT *Event, OUT UINTN *Index);
    wait_for_event: unsafe extern "win64" fn(usize, *const Event, *mut usize) -> Status,
    signal_event: *const NotYetDef,
    close_event: *const NotYetDef,
    check_event: *const NotYetDef,
    install_protocol_interface: *const NotYetDef,
    reinstall_protocol_interface: *const NotYetDef,
    uninstall_protocol_interface: *const NotYetDef,
    handle_protocol: unsafe extern "win64" fn(Handle, *const guid::Guid, &mut *mut CVoid) -> Status,
    __reserved: *const NotYetDef,
    register_protocol_notify: *const NotYetDef,
    locate_handle: *const NotYetDef,
    locate_device_path: *const NotYetDef,
    install_configuration_table: *const NotYetDef,
    load_image: *const NotYetDef,
    start_image: *const NotYetDef,
    exit: *const NotYetDef,
    unload_image: *const NotYetDef,
    exit_boot_services: unsafe extern "win64" fn(image_handle: Handle, map_key: usize) -> Status,
    get_next_monotonic_count: *const NotYetDef,
    stall: unsafe extern "win64" fn(usize) -> Status,
    set_watchdog_timer: unsafe extern "win64" fn(timeout: usize, code: u64, data_size: usize, data: *const u16) -> Status,
    connect_controller: *const NotYetDef,
    disconnect_controller: *const NotYetDef,
    open_protocol: *const NotYetDef,
    close_protocol: unsafe extern "win64" fn(handle: Handle, protocol: *const guid::Guid, agent_handle: Handle, controller_handle: Handle) -> Status,
    open_protocol_information: *const NotYetDef,
    protocols_per_handle: *const NotYetDef,
    locate_handle_buffer: unsafe extern "win64" fn(search_type: LocateSearchType, protocol: *const guid::Guid, search_key: *const CVoid, nhandles: *mut usize, handles: *mut *mut CVoid) -> Status,
    locate_protocol: unsafe extern "win64" fn(protocol: *const guid::Guid, registration: *mut CVoid, interface: *mut *mut CVoid) -> Status,
    install_multiple_protocol_interfaces: *const NotYetDef,
    uninstall_multiple_protocol_interfaces: *const NotYetDef,
    calculate_crc32: *const NotYetDef,
    copy_mem: unsafe extern "win64" fn(*mut CVoid, *mut CVoid, usize),
    set_mem: unsafe extern "win64" fn(*mut CVoid, usize, u8),
    create_event_ex: *const NotYetDef,
}

impl BootServices {
    // return (memory_map, memory_map_size, map_key, descriptor_size, descriptor_version)
    pub unsafe fn get_memory_map(&self, memory_map_size: &mut usize)
                          -> Result<(&'static MemoryDescriptor, usize, usize, usize, u32), Status> {
        let ptr = try!(self.allocate_pool::<MemoryDescriptor>(*memory_map_size));
        let mut map_key: usize = 0;
        let mut descriptor_size: usize = 0;
        let mut descriptor_version: u32 = 0;

        let status = (self.get_memory_map)(memory_map_size, ptr, &mut map_key,
                                  &mut descriptor_size, &mut descriptor_version);
        if status == Status::Success {
            let r = mem::transmute::<*mut MemoryDescriptor, &'static MemoryDescriptor>(ptr);
            Ok((r, map_key, *memory_map_size, descriptor_size,descriptor_version))
            
        } else {
            self.free_pool::<MemoryDescriptor>(ptr);
            Err(status)
        }
    }

    pub fn allocate_pool<T>(&self, buffer_size: usize) -> Result<*mut T, Status>{
        let mut ptr: *mut CVoid = 0 as *mut CVoid;
        let status = unsafe { (self.allocate_pool)(::get_pool_allocation_type(), buffer_size, &mut ptr) };
        if status != Status::Success {
            return Err(status);
        }
        Ok(ptr as *mut T)
    }

    pub fn free_pool<T>(&self, p: *const T) {
        unsafe {
            (self.free_pool)(p as *mut CVoid);
        }
    }

    pub fn wait_for_event(&self, events: &[Event]) -> Result<usize, Status> {
        // XXX: asserts sizeof *Cvoid == sizeof Event
        if false {
            use core::mem;
            unsafe { mem::transmute::<*const CVoid, Event>(0x1 as *const CVoid); }
        }

        let mut index : usize = 0;

        let result = unsafe { (self.wait_for_event)(events.len(), events.as_ptr(), &mut index) };
        if result != Status::Success {
            return Err(result);
        }

        Ok(index)
    }

    pub fn handle_protocol<T: protocols::Protocol>(&self, handle: &Handle) -> Result<&'static T, Status> {
        let mut ptr : *mut CVoid = 0 as *mut CVoid;
        let guid = T::guid();


        unsafe {
            let status = (self.handle_protocol)(*handle, guid, &mut ptr);
            if status != Status::Success {
                return Err(status);
            }
        }

        let r = unsafe { mem::transmute::<*mut CVoid, &'static T>(ptr) };
        Ok(r)
    }

    // TODO: for the love of types, fix me
    pub fn close_protocol<T: protocols::Protocol>(&self, handle: Handle, agent_handle: Handle, controller_handle: Handle) -> Status {
        let guid = T::guid();

        unsafe {
            (self.close_protocol)(handle, guid, agent_handle, controller_handle)
        }
    }

    pub fn locate_protocol<T: protocols::Protocol>(&self) -> Result<&'static T, Status> {
        let guid = T::guid();
        let ptr : *mut CVoid = ptr::null_mut();
        let mut interface = try!(self.allocate_pool::<T>(mem::size_of::<T>()));

        let status = unsafe {
            (self.locate_protocol)(guid, ptr, mem::transmute::<&mut *mut T, *mut *mut CVoid>(&mut interface))
        };

        if status == Status::Success {
            unsafe{ Ok(mem::transmute::<*mut T, &'static T>(interface)) }
        } else {
            self.free_pool::<T>(interface);
            Err(status)
        }
    }

    /// Retrives a slice of handles by protocol GUID.
    pub fn locate_handle_by_protocol<T: protocols::Protocol>(&self) -> Result<Handles, Status> {
        let mut nhandles : usize = 0;
        let mut handles : *mut CVoid = ptr::null_mut();
        let guid = T::guid();

        let res = unsafe { (self.locate_handle_buffer)(LocateSearchType::ByProtocol, guid, ptr::null(), &mut nhandles as *mut usize, &mut handles) };

        if res != Status::Success {
            return Err(res);
        }

        return Ok(Handles::new(handles as *mut Handle, nhandles));
    }

    pub fn exit_boot_services(&self, image_handle: &Handle, map_key: &usize) -> Status {
        unsafe {
            (self.exit_boot_services)(*image_handle, *map_key)
        }
    }

    /// Sleep for a number of microseconds.
    pub fn stall(&self, microseconds: usize) {
        unsafe {
            (self.stall)(microseconds);
        }
    }

    /// Set or disable the watchdog timer.
    pub fn set_watchdog_timer(&self, seconds: usize, code: u64) -> Status {
        unsafe {
            (self.set_watchdog_timer)(seconds, code, 0, ptr::null())
        }
    }

    /// Copy memory, similar to memcpy.
    pub fn copy_mem(&self, dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
        unsafe {
            (self.copy_mem)(dest as *mut CVoid, src as *mut CVoid, n);
        }

        return dest;
    }

    /// Set memory, similar to memset.
    pub fn set_mem(&self, s: *mut u8, c: u8, n: usize) -> *mut u8 {
        unsafe {
            (self.set_mem)(s as *mut CVoid, n, c);
        }

        return s;
    }
}

