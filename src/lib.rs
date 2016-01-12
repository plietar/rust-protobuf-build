extern crate libc;
extern crate protobuf;

use libc::c_void;
use libc::size_t;
use protobuf::descriptor::FileDescriptorProto;
use protobuf::codegen;
use std::io::Write;
use std::ffi::CString;
use std::fs::File;
use std::path::Path;
use std::path::PathBuf;

mod ffi {
    use libc::c_char;
    use libc::c_void;
    use libc::size_t;

    pub enum DiskSourceTree {}
    pub enum ConsoleErrorCollector {}
    pub enum Importer {}

    extern "C" {
        pub fn DiskSourceTree_new() -> *mut DiskSourceTree;
        pub fn DiskSourceTree_MapPath(this: *mut DiskSourceTree,
                                      virtual_path: *const c_char,
                                      disk_path: *const c_char);
        pub fn DiskSourceTree_delete(this: *mut DiskSourceTree);

        pub fn ConsoleErrorCollector_new() -> *mut ConsoleErrorCollector;
        pub fn ConsoleErrorCollector_delete(this: *mut ConsoleErrorCollector);

        pub fn Importer_new(source_tree: *const DiskSourceTree,
                            error_collector: *const ConsoleErrorCollector)
                            -> *mut Importer;
        pub fn Importer_Import(this: *mut Importer,
                               filename: *const c_char,
                               decode_fn: extern "C" fn(c_data: *const u8, data_size: size_t)
                                                        -> *mut c_void)
                               -> *mut c_void;
        pub fn Importer_delete(this: *mut Importer);
    }
}

pub struct DiskSourceTree(*mut ffi::DiskSourceTree);
pub struct ConsoleErrorCollector(*mut ffi::ConsoleErrorCollector);
pub struct Importer(*mut ffi::Importer, DiskSourceTree, ConsoleErrorCollector);

impl DiskSourceTree {
    pub fn new() -> DiskSourceTree {
        unsafe { DiskSourceTree(ffi::DiskSourceTree_new()) }
    }

    pub fn map_path(&mut self, virtual_path: &str, disk_path: &str) {
        let virtual_path = CString::new(virtual_path).unwrap();
        let disk_path = CString::new(disk_path).unwrap();

        unsafe {
            ffi::DiskSourceTree_MapPath(self.0, virtual_path.as_ptr(), disk_path.as_ptr());
        }
    }
}

impl Drop for DiskSourceTree {
    fn drop(&mut self) {
        unsafe {
            ffi::DiskSourceTree_delete(self.0);
        }
    }
}

impl ConsoleErrorCollector {
    pub fn new() -> ConsoleErrorCollector {
        unsafe { ConsoleErrorCollector(ffi::ConsoleErrorCollector_new()) }
    }
}

impl Drop for ConsoleErrorCollector {
    fn drop(&mut self) {
        unsafe {
            ffi::ConsoleErrorCollector_delete(self.0);
        }
    }
}

impl Importer {
    pub fn new(source_tree: DiskSourceTree, error_collector: ConsoleErrorCollector) -> Importer {
        unsafe {
            Importer(ffi::Importer_new(source_tree.0, error_collector.0),
                     source_tree,
                     error_collector)
        }
    }

    extern "C" fn parse_file_descriptor(c_data: *const u8, data_size: size_t) -> *mut c_void {
        let data = unsafe { std::slice::from_raw_parts(c_data, data_size) };

        let desc: FileDescriptorProto = protobuf::parse_from_bytes(data).unwrap();

        Box::into_raw(Box::new(desc)) as *mut c_void
    }

    pub fn import(&mut self, filename: &str) -> Result<FileDescriptorProto, ()> {
        let filename = CString::new(filename).unwrap();

        unsafe {
            let ptr = ffi::Importer_Import(self.0,
                                           filename.as_ptr(),
                                           Importer::parse_file_descriptor);
            if ptr.is_null() {
                return Err(())
            }

            Ok(*Box::from_raw(ptr as *mut FileDescriptorProto))
        }
    }
}

impl Drop for Importer {
    fn drop(&mut self) {
        unsafe {
            ffi::Importer_delete(self.0);
        }
    }
}

pub struct Compiler {
    importer: Importer,
    output_path: PathBuf,
}

impl Compiler {
    pub fn new<P1: AsRef<Path>, P2: AsRef<Path>>(input_path: P1, output_path: P2) -> Compiler {
        let mut source_tree = DiskSourceTree::new();
        source_tree.map_path("", input_path.as_ref().to_str().unwrap());

        let error_collector = ConsoleErrorCollector::new();

        let importer = Importer::new(source_tree, error_collector);

        Compiler {
            importer: importer,
            output_path: output_path.as_ref().to_owned(),
        }
    }

    pub fn compile(&mut self, filename: &str) -> Result<(), ()> {
        let desc = try!(self.importer.import(filename));

        let name = desc.get_name().to_owned();
        let results = codegen::gen(&[desc], &[name]);

        for result in results {
            let out_path = self.output_path.join(result.name);
            let mut out_file = File::create(out_path).unwrap();
            out_file.write_all(&result.content).unwrap();
        }

        Ok(())
    }
}
