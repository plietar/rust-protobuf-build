#include <iostream>
#include <google/protobuf/descriptor.h>
#include <google/protobuf/descriptor.pb.h>
#include <google/protobuf/compiler/importer.h>
#include <google/protobuf/compiler/code_generator.h>
#include <google/protobuf/compiler/command_line_interface.h>

using namespace google::protobuf;
using namespace google::protobuf::compiler;

class ConsoleErrorCollector : public MultiFileErrorCollector {
    public:
        virtual void AddError(const std::string &filename, int line, int column, const std::string &message) {
            std::cerr << filename << ":" << line + 1 << ":" << column + 1 << ": " << message << std::endl;
        }

        virtual ~ConsoleErrorCollector() {}
};

extern "C" {
    DiskSourceTree *DiskSourceTree_new() {
        return new DiskSourceTree();
    }

    void DiskSourceTree_MapPath(DiskSourceTree *thiz, const char * virtual_path, const char * disk_path) {
        thiz->MapPath(virtual_path, disk_path);
    }

    void DiskSourceTree_delete(DiskSourceTree *thiz) {
        delete thiz;
    }

    ConsoleErrorCollector *ConsoleErrorCollector_new() {
        return new ConsoleErrorCollector();
    }

    void ConsoleErrorCollector_delete(ConsoleErrorCollector *thiz) {
        delete thiz;
    }

    Importer *Importer_new(DiskSourceTree *source_tree, ConsoleErrorCollector *error_collector) {
        return new Importer(source_tree, error_collector);
    }

    void *Importer_Import(Importer *thiz, const char *filename, void* (*decode_fn)(const uint8_t *, size_t)) {
        const FileDescriptor *file = thiz->Import(std::string(filename));
        if (file == NULL) {
            return NULL;
        }

        FileDescriptorProto desc;
        file->CopyTo(&desc);

        // Missing in old versions of libprotoc
        // Not really required by rust-protobuf anyway
        //file->CopyJsonNameTo(&desc);
        //file->CopySourceCodeInfoTo(&desc);

        std::string out;
        desc.SerializeToString(&out);

        return decode_fn((const uint8_t*) out.c_str(), out.length());
    }

    void Importer_delete(Importer *thiz) {
        delete thiz;
    }
}

