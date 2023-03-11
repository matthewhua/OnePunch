package io.matt.struct.briage.fileupload;

import java.io.File;

public class FileUploaderImpl implements FileUploader {

    private FileUploadExecutor executor = null;

    public FileUploaderImpl(FileUploadExecutor executor) {
        this.executor = executor;
    }

    @Override
    public Object upload(String path) {
        return executor.uploadFile(path);
    }

    @Override
    public boolean check(Object object) {
        return executor.checkFile(object);
    }
}
